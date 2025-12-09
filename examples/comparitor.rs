use assert2::check;
use cfb::CompoundFile;
use clap::Parser;
use itertools::Itertools;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    io::{Read, Seek},
    path::PathBuf,
    rc::Rc,
};
use msi::{
    Column, PackageType, Rows, SummaryInfo, Table,
    internal::stringpool::{StringPool, StringPoolBuilder},
};

macro_rules! print_assert_eq {
    // 1. Handle case with custom message arguments
    ($left:expr, $right:expr, $($arg:tt)+) => ({
        match (&$left, &$right) {
            (left_val, right_val) => {
                if *left_val != *right_val {
                    std::eprintln!(
                        "Assertion Failed: `(left == right)`\n  left: `{:?}`,\n right: `{:?}`",
                        left_val,
                        right_val
                    );
                    std::eprintln!($($arg)+); // Print custom message
                    std::eprintln!("\n\n");
                }
            }
        }
    });
    // 2. Handle case with no custom message
    ($left:expr, $right:expr) => ({
        match (&$left, &$right) {
            (left_val, right_val) => {
                if *left_val != *right_val {
                    std::eprintln!(
                        "Assertion Failed: `(left == right)`\n  left: `{:?}`,\n right: `{:?}`",
                        left_val,
                        right_val
                    );
                    std::eprintln!("\n\n");
                }
            }
        }
    });
}

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    first: PathBuf,
    second: PathBuf,
}

fn main() {
    println!("Comparing MSIs");
    let args = Args::parse();
    let mut comp1 = cfb::CompoundFile::open_strict(
        std::fs::File::open(args.first.clone()).unwrap(),
    )
    .unwrap();
    let mut comp2 = cfb::CompoundFile::open_strict(
        std::fs::File::open(args.second.clone()).unwrap(),
    )
    .unwrap();

    print_assert_eq!(
        get_package_type(&mut comp1),
        get_package_type(&mut comp2)
    );
    compare_summary_info(&mut comp1, &mut comp2);

    let string_pool1 = get_string_pool(&mut comp1);
    let string_pool2 = get_string_pool(&mut comp2);

    let (table_names1, mut all_tables1) =
        get_table_info(&string_pool1, &mut comp1);
    let (table_names2, mut all_tables2) =
        get_table_info(&string_pool2, &mut comp2);

    let missing_table_names1 = table_names2.difference(&table_names1);
    print_assert_eq!(
        missing_table_names1.clone().count(),
        0,
        "Tables missing from {:?}: {:?}",
        args.first,
        missing_table_names1
    );
    let missing_table_names2 = table_names1.difference(&table_names2);
    print_assert_eq!(
        missing_table_names2.clone().count(),
        0,
        "Tables missing from {:?}: {:?}",
        args.second,
        missing_table_names2
    );

    let columns1 = get_column_info(
        &table_names1,
        &string_pool1,
        &mut comp1,
        &mut all_tables1,
    );
    let columns2 = get_column_info(
        &table_names2,
        &string_pool2,
        &mut comp2,
        &mut all_tables2,
    );
    let shared_table_names =
        table_names1.intersection(&table_names2).collect_vec();
    compare_column_info(shared_table_names, columns1, columns2);
}

fn get_package_type<F: Read + Seek>(comp: &mut CompoundFile<F>) -> PackageType {
    let root_entry = comp.root_entry();
    let clsid = root_entry.clsid();
    match PackageType::from_clsid(clsid) {
        Some(ptype) => ptype,
        None => panic!("Unrecognized package CLSID ({})", clsid.hyphenated()),
    }
}

fn compare_summary_info<F: Read + Seek>(
    comp: &mut CompoundFile<F>,
    comp2: &mut CompoundFile<F>,
) {
    pub const SUMMARY_INFO_STREAM_NAME: &str = "\u{5}SummaryInformation";
    let mut summary_info =
        SummaryInfo::read(comp.open_stream(SUMMARY_INFO_STREAM_NAME).unwrap())
            .unwrap();
    let mut summary_info2 =
        SummaryInfo::read(comp2.open_stream(SUMMARY_INFO_STREAM_NAME).unwrap())
            .unwrap();

    // There are 19 properties possible in the summary info stream.
    for i in 1..19 {
        print_assert_eq!(
            summary_info.properties_mut().get(i),
            summary_info2.properties_mut().get(i),
            "Property {} is different",
            i
        );
    }
}

fn get_string_pool<F: Read + Seek>(comp: &mut CompoundFile<F>) -> StringPool {
    let builder = {
        const STRING_POOL_TABLE_NAME: &str = "_StringPool";
        let name = msi::internal::streamname::encode(
            STRING_POOL_TABLE_NAME,
            true,
        );
        let stream = comp.open_stream(name).unwrap();
        StringPoolBuilder::read_from_pool(stream).unwrap()
    };
    const STRING_DATA_TABLE_NAME: &str = "_StringData";
    let name =
        msi::internal::streamname::encode(STRING_DATA_TABLE_NAME, true);
    let stream = comp.open_stream(name).unwrap();
    builder.build_from_data(stream).unwrap()
}

/// Gets the tables present in the given MSI
fn get_table_info<F: Read + Seek>(
    string_pool: &StringPool,
    comp: &mut CompoundFile<F>,
) -> (HashSet<String>, BTreeMap<String, Rc<Table>>) {
    let mut all_tables = BTreeMap::<String, Rc<Table>>::new();
    const TABLES_TABLE_NAME: &str = "_Tables";
    fn make_tables_table(long_string_refs: bool) -> Rc<Table> {
        Table::new(
            TABLES_TABLE_NAME.to_string(),
            vec![Column::build("Name").primary_key().string(64)],
            long_string_refs,
        )
    }
    // Read in _Tables table:
    let table = make_tables_table(string_pool.long_string_refs());
    let stream_name = table.stream_name();
    let mut names = HashSet::<String>::new();
    if comp.exists(&stream_name) {
        let stream = comp.open_stream(&stream_name).unwrap();
        let rows = Rows::new(
            string_pool,
            table.clone(),
            table.read_rows(stream).unwrap(),
        );
        for row in rows {
            let table_name = row[0].as_str().unwrap().to_string();
            if names.contains(&table_name) {
                panic!(
                    "Repeated key in {:?} table: {:?}",
                    TABLES_TABLE_NAME, table_name
                );
            }
            names.insert(table_name);
        }
    }
    all_tables.insert(table.name().to_string(), table);
    (names, all_tables)
}

// Gets the column information for the given table in the given MSI
fn get_column_info<F: Read + Seek>(
    table_names: &HashSet<String>,
    string_pool: &StringPool,
    comp: &mut CompoundFile<F>,
    all_tables: &mut BTreeMap<String, Rc<Table>>,
) -> HashMap<String, BTreeMap<i32, (String, i32)>> {
    // Read in _Columns table:
    let mut columns_map: HashMap<String, BTreeMap<i32, (String, i32)>> =
        table_names
            .into_iter()
            .map(|name| (name.clone(), BTreeMap::new()))
            .collect();
    {
        const COLUMNS_TABLE_NAME: &str = "_Columns";
        fn make_columns_table(long_string_refs: bool) -> Rc<Table> {
            Table::new(
                COLUMNS_TABLE_NAME.to_string(),
                vec![
                    Column::build("Table").primary_key().string(64),
                    Column::build("Number").primary_key().int16(),
                    Column::build("Name").string(64),
                    Column::build("Type").int16(),
                ],
                long_string_refs,
            )
        }
        let table = make_columns_table(string_pool.long_string_refs());
        let stream_name = table.stream_name();
        if comp.exists(&stream_name) {
            let stream = comp.open_stream(&stream_name).unwrap();
            let rows = Rows::new(
                string_pool,
                table.clone(),
                table.read_rows(stream).unwrap(),
            );
            for row in rows {
                let table_name = row[0].as_str().unwrap();
                if let Some(cols) = columns_map.get_mut(table_name) {
                    let col_index = row[1].as_int().unwrap();
                    if cols.contains_key(&col_index) {
                        panic!(
                            "Repeated key in {:?} table: {:?}",
                            COLUMNS_TABLE_NAME,
                            (table_name, col_index)
                        );
                    }
                    let col_name = row[2].as_str().unwrap().to_string();
                    let type_bits = row[3].as_int().unwrap();
                    cols.insert(col_index, (col_name, type_bits));
                } else {
                    panic!(
                        "_Columns mentions table {:?}, which isn't in \
                             _Tables",
                        table_name
                    );
                }
            }
        }
        all_tables.insert(table.name().to_string(), table);
    }
    columns_map
}

fn compare_column_info(
    shared_tables: Vec<&String>,
    column1: HashMap<String, BTreeMap<i32, (String, i32)>>,
    column2: HashMap<String, BTreeMap<i32, (String, i32)>>,
) {
    for table in shared_tables {
        let column1 = column1.get(table).unwrap();
        let column2 = column2.get(table).unwrap();
        print_assert_eq!(
            column1.keys().count(),
            column2.keys().count(),
            "Number of table columns for table {} is different",
            table
        );
        let columns1 = column1.values().collect_vec();
        let columns2 = column2.values().collect_vec();
        for i in 0..columns1.len().min(columns2.len()) {
            let (index1, type1) = columns1.get(i).unwrap();
            let (index2, type2) = columns2.get(i).unwrap();
            print_assert_eq!(
                index1,
                index2,
                "Indices for column in table [{table}] are out of order."
            );
            print_assert_eq!(
                type1,
                type2,
                "Types in table {table} column {i} are different"
            );
        }
    }
}

// // Read in _Validation table:
// let mut validation_map =
//     HashMap::<(String, String), Vec<ValueRef>>::new();
// {
//     let table = make_validation_table(string_pool.long_string_refs());
//     // TODO: Ensure that columns_map["_Validation"].columns() matches
//     // the hard-coded validation table definition.
//     let stream_name = table.stream_name();
//     if comp.exists(&stream_name) {
//         let stream = comp.open_stream(&stream_name)?;
//         for value_refs in table.read_rows(stream)? {
//             let table_name = value_refs[0]
//                 .to_value(&string_pool)
//                 .as_str()
//                 .unwrap()
//                 .to_string();
//             let column_name = value_refs[1]
//                 .to_value(&string_pool)
//                 .as_str()
//                 .unwrap()
//                 .to_string();
//             let key = (table_name, column_name);
//             if validation_map.contains_key(&key) {
//                 invalid_data!(
//                     "Repeated key in {:?} table: {:?}",
//                     VALIDATION_TABLE_NAME,
//                     key
//                 );
//             }
//             validation_map.insert(key, value_refs);
//         }
//     }
// }
// // Construct Table objects from column/validation data:
// for (table_name, column_specs) in columns_map {
//     if column_specs.is_empty() {
//         invalid_data!("No columns found for table {:?}", table_name);
//     }
//     let num_columns = column_specs.len() as i32;
//     if column_specs.keys().next() != Some(&1)
//         || column_specs.keys().next_back() != Some(&num_columns)
//     {
//         invalid_data!(
//             "Table {:?} does not have a complete set of columns",
//             table_name
//         );
//     }
//     let mut columns = Vec::<Column>::with_capacity(column_specs.len());
//     for (_, (column_name, bitfield)) in column_specs {
//         let mut builder = Column::build(column_name.as_str());
//         let key = (table_name.clone(), column_name);
//         if let Some(value_refs) = validation_map.get(&key) {
//             let is_nullable = value_refs[2].to_value(&string_pool);
//             if is_nullable.as_str().unwrap() == "Y" {
//                 builder = builder.nullable();
//             }
//             let min_value = value_refs[3].to_value(&string_pool);
//             let max_value = value_refs[4].to_value(&string_pool);
//             if !min_value.is_null() && !max_value.is_null() {
//                 let min = min_value.as_int().unwrap();
//                 let max = max_value.as_int().unwrap();
//                 builder = builder.range(min, max);
//             }
//             let key_table = value_refs[5].to_value(&string_pool);
//             let key_column = value_refs[6].to_value(&string_pool);
//             if !key_table.is_null() && !key_column.is_null() {
//                 builder = builder.foreign_key(
//                     key_table.as_str().unwrap(),
//                     key_column.as_int().unwrap(),
//                 );
//             }
//             let category_value = value_refs[7].to_value(&string_pool);
//             if !category_value.is_null() {
//                 let category = category_value
//                     .as_str()
//                     .unwrap()
//                     .parse::<Category>()
//                     .ok();
//                 if let Some(category) = category {
//                     builder = builder.category(category);
//                 }
//             }
//             let enum_values = value_refs[8].to_value(&string_pool);
//             if !enum_values.is_null() {
//                 let enum_values: Vec<&str> =
//                     enum_values.as_str().unwrap().split(';').collect();
//                 builder = builder.enum_values(&enum_values);
//             }
//         }
//         columns.push(builder.with_bitfield(bitfield)?);
//     }
//     let table =
//         Table::new(table_name, columns, string_pool.long_string_refs());
//     all_tables.insert(table.name().to_string(), table);
// }
