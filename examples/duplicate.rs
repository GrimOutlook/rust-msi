use std::fs::File;

use clap::Parser;
use clio::ClioPath;
use msi::Value;
use uuid::Uuid;

/// Simple minimum reproducible example showing that a base MSI becomes invalid
/// after writing a property to the file.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Filename of the MSI to create
    #[arg(value_parser = clap::value_parser!(ClioPath), default_value = "mre.msi")]
    output: ClioPath,

    /// Revision Number (SummaryInformation property #9) to use for the generated MSI.
    /// `msitools` doesn't provide a way to set this so during testing, when
    /// trying to make the MSIs as similar as possible, we may want to set this
    /// based on external information (e.g an `msitools` generated MSI)
    #[arg(long, short ,value_parser = clap::value_parser!(Uuid), default_value_t = Uuid::new_v4())]
    revision_number: Uuid,
}

fn main() {
    let args = Args::parse();

    let file = File::options()
        .create(true)
        .read(true)
        .write(true)
        .truncate(true)
        .open(args.output.to_path_buf())
        .unwrap_or_else(|_| panic!("Failed to open new MSI: {}", args.output));

    let mut package = msi::Package::create(msi::PackageType::Installer, file)
        .expect("Failed to create MSI");

    package.set_database_codepage(msi::CodePage::from_id(0).unwrap());

    // Summary Information
    let sum = package.summary_info_mut();
    sum.set_codepage(msi::CodePage::Windows1252);
    sum.set_arch("Intel");
    sum.set_uuid(args.revision_number); // PID 9
    sum.set_word_count(2);
    sum.set_creating_application("msitools 0.106");
    sum.set_page_count(200);
    sum.set_languages(&[msi::Language::from_code(1033)]);
    sum.set_keywords(&["Installer".to_string(), "0.0.0".to_string()]);
    sum.set_subject("MyDescription");
    sum.set_comments("MyComments");
    sum.set_doc_security(2);
    sum.set_author("MyManufacturer");
    sum.set_creation_time_to_now();
    sum.set_last_save_time_to_now();
    package.flush().unwrap();

    // Tables
    // NOTE: Table insertions are ordered to align with the output from
    // `msitools`

    // ActionText Table
    package
        .create_table(
            "ActionText",
            vec![
                msi::Column::build("Action").primary_key().id_string(72),
                msi::Column::build("Description")
                    .localizable()
                    .nullable()
                    .text_string(255),
                msi::Column::build("Template")
                    .localizable()
                    .nullable()
                    .category(msi::Category::Template)
                    .string(255),
            ],
        )
        .unwrap();

    // NOTE: No rows for ActionText table

    // AdminExecuteSequence table
    package
        .create_table(
            "AdminExecuteSequence",
            vec![
                msi::Column::build("Action").primary_key().id_string(72),
                msi::Column::build("Condition")
                    .nullable()
                    .category(msi::Category::Condition)
                    .string(255),
                msi::Column::build("Sequence").nullable().int16(),
            ],
        )
        .unwrap();
    package
        .insert_rows(msi::Insert::into("AdminExecuteSequence").rows(vec![
            vec![Value::from("CostInitialize"), Value::Null, Value::from(800)],
            vec![Value::from("FileCost"), Value::Null, Value::from(900)],
            vec![Value::from("CostFinalize"), Value::Null, Value::from(1000)],
            vec![
                Value::from("InstallValidate"),
                Value::Null,
                Value::from(1400),
            ],
            vec![
                Value::from("InstallInitialize"),
                Value::Null,
                Value::from(1500),
            ],
            vec![
                Value::from("InstallFinalize"),
                Value::Null,
                Value::from(6600),
            ],
            vec![
                Value::from("InstallAdminPackage"),
                Value::Null,
                Value::from(3900),
            ],
            vec![Value::from("InstallFiles"), Value::Null, Value::from(4000)],
        ]))
        .unwrap();

    // AdminUISequence
    package
        .create_table(
            "AdminUISequence",
            vec![
                msi::Column::build("Action").primary_key().id_string(72),
                msi::Column::build("Condition")
                    .nullable()
                    .category(msi::Category::Condition)
                    .string(255),
                msi::Column::build("Sequence").nullable().int16(),
            ],
        )
        .unwrap();
    package
        .insert_rows(msi::Insert::into("AdminUISequence").rows(vec![
            vec![Value::from("CostInitialize"), Value::Null, Value::from(800)],
            vec![Value::from("FileCost"), Value::Null, Value::from(900)],
            vec![Value::from("CostFinalize"), Value::Null, Value::from(1000)],
            vec![Value::from("ExecuteAction"), Value::Null, Value::from(1300)],
        ]))
        .unwrap();

    // AdvtExecuteSequence
    package
        .create_table(
            "AdvtExecuteSequence",
            vec![
                msi::Column::build("Action").primary_key().id_string(72),
                msi::Column::build("Condition")
                    .nullable()
                    .category(msi::Category::Condition)
                    .string(255),
                msi::Column::build("Sequence").nullable().int16(),
            ],
        )
        .unwrap();
    package
        .insert_rows(msi::Insert::into("AdvtExecuteSequence").rows(vec![
            vec![Value::from("CostInitialize"), Value::Null, Value::from(800)],
            vec![Value::from("CostFinalize"), Value::Null, Value::from(1000)],
            vec![
                Value::from("InstallValidate"),
                Value::Null,
                Value::from(1400),
            ],
            vec![
                Value::from("InstallInitialize"),
                Value::Null,
                Value::from(1500),
            ],
            vec![
                Value::from("PublishFeatures"),
                Value::Null,
                Value::from(6300),
            ],
            vec![
                Value::from("PublishProduct"),
                Value::Null,
                Value::from(6400),
            ],
            vec![
                Value::from("InstallFinalize"),
                Value::Null,
                Value::from(6600),
            ],
        ]))
        .unwrap();

    // AppSearch Table
    package
        .create_table(
            "AppSearch",
            vec![
                msi::Column::build("Property").primary_key().id_string(72),
                msi::Column::build("Signature_").primary_key().id_string(72),
            ],
        )
        .unwrap();

    // Binary Table
    package
        .create_table(
            "Binary",
            vec![
                msi::Column::build("Name").primary_key().id_string(72),
                msi::Column::build("Data").binary(),
            ],
        )
        .unwrap();

    // Component Table
    package
        .create_table(
            "Component",
            vec![
                msi::Column::build("Component").primary_key().id_string(72),
                msi::Column::build("ComponentId")
                    .nullable()
                    .category(msi::Category::Guid)
                    .string(38),
                msi::Column::build("Directory_").id_string(72),
                msi::Column::build("Attributes").int16(),
                msi::Column::build("Condition")
                    .nullable()
                    .category(msi::Category::Condition)
                    .string(255),
                msi::Column::build("KeyPath").nullable().id_string(72),
            ],
        )
        .unwrap();

    // Condition Table
    package
        .create_table(
            "Condition",
            vec![
                msi::Column::build("Feature_").primary_key().id_string(38),
                msi::Column::build("Level").primary_key().int16(),
                msi::Column::build("Condition")
                    .nullable()
                    .category(msi::Category::Condition)
                    .string(255),
            ],
        )
        .unwrap();

    // CreateFolder Table
    package
        .create_table(
            "CreateFolder",
            vec![
                msi::Column::build("Directory_").primary_key().id_string(72),
                msi::Column::build("Component_").primary_key().id_string(72),
            ],
        )
        .unwrap();

    // CustomAction Table
    package
        .create_table(
            "CustomAction",
            vec![
                msi::Column::build("Action").primary_key().id_string(72),
                msi::Column::build("Type").int16(),
                msi::Column::build("Source")
                    .nullable()
                    .category(msi::Category::CustomSource)
                    .string(72),
                msi::Column::build("Target").nullable().formatted_string(255),
                msi::Column::build("ExtendedType").nullable().int32(),
            ],
        )
        .unwrap();

    // Directory Table
    package
        .create_table(
            "Directory",
            vec![
                msi::Column::build("Directory").primary_key().id_string(72),
                msi::Column::build("Directory_Parent")
                    .nullable()
                    .id_string(72),
                msi::Column::build("DefaultDir")
                    .localizable()
                    .category(msi::Category::DefaultDir)
                    .string(255),
            ],
        )
        .unwrap();

    // Environment Table
    package
        .create_table(
            "Environment",
            vec![
                msi::Column::build("Environment").primary_key().id_string(72),
                msi::Column::build("Name").localizable().text_string(64),
                msi::Column::build("Value")
                    .localizable()
                    .nullable()
                    .formatted_string(255),
                msi::Column::build("Component_").id_string(72),
            ],
        )
        .unwrap();

    // Error Table
    package
        .create_table(
            "Error",
            vec![
                msi::Column::build("Error").primary_key().int16(),
                msi::Column::build("Message")
                    .localizable()
                    .nullable()
                    .text_string(0),
            ],
        )
        .unwrap();

    // Feature
    package
        .create_table(
            "Feature",
            vec![
                msi::Column::build("Feature").primary_key().id_string(38),
                msi::Column::build("Feature_Parent").nullable().id_string(38),
                msi::Column::build("Title")
                    .localizable()
                    .nullable()
                    .formatted_string(64),
                msi::Column::build("Description")
                    .localizable()
                    .nullable()
                    .formatted_string(255),
                msi::Column::build("Display").nullable().int16(),
                msi::Column::build("Level").int16(),
                msi::Column::build("Directory_").nullable().id_string(72),
                msi::Column::build("Attributes").int16(),
            ],
        )
        .unwrap();

    // FeatureComponents
    package
        .create_table(
            "FeatureComponents",
            vec![
                msi::Column::build("Feature_").primary_key().id_string(38),
                msi::Column::build("Component_").primary_key().id_string(72),
            ],
        )
        .unwrap();

    // File
    package
        .create_table(
            "File",
            vec![
                msi::Column::build("File").primary_key().id_string(72),
                msi::Column::build("Component_").id_string(72),
                msi::Column::build("FileName")
                    .localizable()
                    .category(msi::Category::Filename)
                    .string(255),
                msi::Column::build("FileSize").int32(),
                msi::Column::build("Version")
                    .nullable()
                    .category(msi::Category::Version)
                    .string(72),
                msi::Column::build("Language")
                    .nullable()
                    .category(msi::Category::Language)
                    .string(20),
                msi::Column::build("Attributes").nullable().int16(),
                msi::Column::build("Sequence").int32(),
            ],
        )
        .unwrap();

    // Icon
    package
        .create_table(
            "Icon",
            vec![
                msi::Column::build("Name").primary_key().id_string(72),
                msi::Column::build("Data").binary(),
            ],
        )
        .unwrap();

    // IniFile
    package
        .create_table(
            "IniFile",
            vec![
                msi::Column::build("IniFile").primary_key().id_string(72),
                msi::Column::build("FileName")
                    .localizable()
                    .category(msi::Category::Filename)
                    .string(255),
                msi::Column::build("DirProperty").nullable().id_string(72),
                msi::Column::build("Section")
                    .localizable()
                    .formatted_string(255),
                msi::Column::build("Key").localizable().formatted_string(255),
                msi::Column::build("Value")
                    .localizable()
                    .formatted_string(255),
                msi::Column::build("Action").int16(),
                msi::Column::build("Component_").id_string(72),
            ],
        )
        .unwrap();

    // InstallExecuteSequence table
    package
        .create_table(
            "InstallExecuteSequence",
            vec![
                msi::Column::build("Action").primary_key().id_string(72),
                msi::Column::build("Condition")
                    .nullable()
                    .category(msi::Category::Condition)
                    .string(255),
                msi::Column::build("Sequence").nullable().int16(),
            ],
        )
        .unwrap();

    package
        .insert_rows(msi::Insert::into("InstallExecuteSequence").rows(vec![
            vec![Value::from("CostInitialize"), Value::Null, Value::from(800)],
            vec![Value::from("FileCost"), Value::Null, Value::from(900)],
            vec![Value::from("CostFinalize"), Value::Null, Value::from(1000)],
            vec![
                Value::from("ValidateProductID"),
                Value::Null,
                Value::from(700),
            ],
            vec![
                Value::from("InstallValidate"),
                Value::Null,
                Value::from(1400),
            ],
            vec![
                Value::from("InstallInitialize"),
                Value::Null,
                Value::from(1500),
            ],
            vec![
                Value::from("ProcessComponents"),
                Value::Null,
                Value::from(1600),
            ],
            vec![
                Value::from("UnpublishedFeatures"),
                Value::Null,
                Value::from(1800),
            ],
            vec![Value::from("RegisterUser"), Value::Null, Value::from(6000)],
            vec![
                Value::from("RegisterProduct"),
                Value::Null,
                Value::from(6100),
            ],
            vec![
                Value::from("PublishFeatures"),
                Value::Null,
                Value::from(6300),
            ],
            vec![
                Value::from("PublishProduct"),
                Value::Null,
                Value::from(6400),
            ],
            vec![
                Value::from("InstallFinalize"),
                Value::Null,
                Value::from(6600),
            ],
        ]))
        .unwrap();

    // InstallUISequence
    package
        .create_table(
            "InstallUISequence",
            vec![
                msi::Column::build("Action").primary_key().id_string(72),
                msi::Column::build("Condition")
                    .nullable()
                    .category(msi::Category::Condition)
                    .string(255),
                msi::Column::build("Sequence").nullable().int16(),
            ],
        )
        .unwrap();
    package
        .insert_rows(msi::Insert::into("InstallUISequence").rows(vec![
            vec![Value::from("CostInitialize"), Value::Null, Value::from(800)],
            vec![Value::from("FileCost"), Value::Null, Value::from(900)],
            vec![Value::from("CostFinalize"), Value::Null, Value::from(1000)],
            vec![Value::from("ExecuteAction"), Value::Null, Value::from(1300)],
            vec![
                Value::from("ValidateProductID"),
                Value::Null,
                Value::from(700),
            ],
        ]))
        .unwrap();

    // LaunchCondition
    package
        .create_table(
            "LaunchCondition",
            vec![
                msi::Column::build("Condition")
                    .primary_key()
                    .category(msi::Category::Condition)
                    .string(255),
                msi::Column::build("Description")
                    .localizable()
                    .formatted_string(255),
            ],
        )
        .unwrap();

    // Media
    package
        .create_table(
            "Media",
            vec![
                msi::Column::build("DiskId").primary_key().int16(),
                msi::Column::build("LastSequence").int32(),
                msi::Column::build("DiskPrompt")
                    .nullable()
                    .localizable()
                    .text_string(64),
                msi::Column::build("Cabinet")
                    .nullable()
                    .category(msi::Category::Cabinet)
                    .string(255),
                msi::Column::build("VolumeLabel").nullable().text_string(32),
                msi::Column::build("Source")
                    .nullable()
                    .category(msi::Category::Property)
                    .string(72),
            ],
        )
        .unwrap();

    // MsiFileHash
    package
        .create_table(
            "MsiFileHash",
            vec![
                msi::Column::build("File_").primary_key().id_string(72),
                msi::Column::build("Options").int16(),
                msi::Column::build("HashPart1").int32(),
                msi::Column::build("HashPart2").int32(),
                msi::Column::build("HashPart3").int32(),
                msi::Column::build("HashPart4").int32(),
            ],
        )
        .unwrap();

    // Property Table
    package
        .create_table(
            "Property",
            vec![
                msi::Column::build("Property").primary_key().id_string(72),
                msi::Column::build("Value").localizable().text_string(0),
            ],
        )
        .unwrap();
    package
        .insert_rows(msi::Insert::into("Property").rows(vec![
            vec![Value::from("UpgradeCode"), Value::from("{*}")],
            vec![Value::from("Manufacturer"), Value::from("MyManufacturer")],
            vec![Value::from("ProductLanguage"), Value::from("1033")],
            vec![
                Value::from("ProductCode"),
                Value::from("{11111111-1111-1111-1111-111111111113}"),
            ],
            vec![Value::from("ProductName"), Value::from("MyProduct")],
            vec![Value::from("ProductVersion"), Value::from("0.0.0")],
        ]))
        .unwrap();

    // RegLocator
    package
        .create_table(
            "RegLocator",
            vec![
                msi::Column::build("Signature_").primary_key().id_string(72),
                msi::Column::build("Root").int16(),
                msi::Column::build("Key")
                    .category(msi::Category::RegPath)
                    .string(255),
                msi::Column::build("Name").nullable().formatted_string(255),
                msi::Column::build("Type").nullable().int16(),
            ],
        )
        .unwrap();

    // Registry
    package
        .create_table(
            "Registry",
            vec![
                msi::Column::build("Registry").primary_key().id_string(72),
                msi::Column::build("Root").int16(),
                msi::Column::build("Key")
                    .localizable()
                    .category(msi::Category::RegPath)
                    .string(255),
                msi::Column::build("Name")
                    .nullable()
                    .localizable()
                    .formatted_string(255),
                msi::Column::build("Value")
                    .nullable()
                    .localizable()
                    .formatted_string(0),
                msi::Column::build("Component_").id_string(72),
            ],
        )
        .unwrap();

    // RemoveFile
    package
        .create_table(
            "RemoveFile",
            vec![
                msi::Column::build("FileKey").primary_key().id_string(72),
                msi::Column::build("Component_").id_string(72),
                msi::Column::build("FileName")
                    .nullable()
                    .localizable()
                    .category(msi::Category::WildCardFilename)
                    .string(255),
                msi::Column::build("DirProperty").id_string(72),
                msi::Column::build("InstallMode").int16(),
            ],
        )
        .unwrap();

    // RemoveIniFile
    package
        .create_table(
            "RemoveIniFile",
            vec![
                msi::Column::build("RemoveIniFile")
                    .primary_key()
                    .id_string(72),
                msi::Column::build("FileName")
                    .localizable()
                    .category(msi::Category::Filename)
                    .string(255),
                msi::Column::build("DirProperty").nullable().id_string(72),
                msi::Column::build("Section")
                    .localizable()
                    .formatted_string(255),
                msi::Column::build("Key").localizable().formatted_string(255),
                msi::Column::build("Value")
                    .nullable()
                    .localizable()
                    .formatted_string(255),
                msi::Column::build("Action").int16(),
                msi::Column::build("Component_").id_string(72),
            ],
        )
        .unwrap();

    // ServiceControl
    package
        .create_table(
            "ServiceControl",
            vec![
                msi::Column::build("ServiceControl")
                    .primary_key()
                    .id_string(72),
                msi::Column::build("Name").localizable().formatted_string(255),
                msi::Column::build("Event").int16(),
                msi::Column::build("Arguments")
                    .nullable()
                    .localizable()
                    .formatted_string(255),
                msi::Column::build("Wait").nullable().int16(),
                msi::Column::build("Component_").id_string(72),
            ],
        )
        .unwrap();

    // ServiceInstall
    package
        .create_table(
            "ServiceInstall",
            vec![
                msi::Column::build("ServiceInstall")
                    .primary_key()
                    .id_string(72),
                msi::Column::build("Name").formatted_string(255),
                msi::Column::build("DisplayName")
                    .nullable()
                    .localizable()
                    .formatted_string(255),
                msi::Column::build("ServiceType").int32(),
                msi::Column::build("StartType").int32(),
                msi::Column::build("ErrorControl").int32(),
                msi::Column::build("LoadOrderGroup")
                    .nullable()
                    .formatted_string(255),
                msi::Column::build("Dependencies")
                    .nullable()
                    .formatted_string(255),
                msi::Column::build("StartName")
                    .nullable()
                    .formatted_string(255),
                msi::Column::build("Password")
                    .nullable()
                    .formatted_string(255),
                msi::Column::build("Arguments")
                    .nullable()
                    .formatted_string(255),
                msi::Column::build("Component_").id_string(72),
                msi::Column::build("Description")
                    .nullable()
                    .localizable()
                    .formatted_string(255),
            ],
        )
        .unwrap();

    // Shortcut
    package
        .create_table(
            "Shortcut",
            vec![
                msi::Column::build("Shortcut").primary_key().id_string(72),
                msi::Column::build("Directory_").id_string(72),
                msi::Column::build("Name")
                    .localizable()
                    .category(msi::Category::Filename)
                    .string(128),
                msi::Column::build("Component_").id_string(72),
                msi::Column::build("Target")
                    .category(msi::Category::Shortcut)
                    .string(72),
                msi::Column::build("Arguments")
                    .nullable()
                    .formatted_string(255),
                msi::Column::build("Description")
                    .nullable()
                    .localizable()
                    .string(255),
                msi::Column::build("Hotkey").nullable().int16(),
                msi::Column::build("Icon_").nullable().id_string(72),
                msi::Column::build("IconIndex").nullable().int16(),
                msi::Column::build("ShowCmd").nullable().int16(),
                msi::Column::build("WkDir").nullable().id_string(72),
                msi::Column::build("DisplayResourceDLL")
                    .nullable()
                    .formatted_string(255),
                msi::Column::build("DisplayResourceId").nullable().int16(),
                msi::Column::build("DescriptionResourceDLL")
                    .nullable()
                    .formatted_string(255),
                msi::Column::build("DescriptionResourceId").nullable().int16(),
            ],
        )
        .unwrap();

    // Signature
    package
        .create_table(
            "Signature",
            vec![
                msi::Column::build("Signature").primary_key().id_string(72),
                msi::Column::build("FileName").text_string(255),
                msi::Column::build("MinVersion").nullable().text_string(20),
                msi::Column::build("MaxVersion").nullable().text_string(20),
                msi::Column::build("MinSize").nullable().int32(),
                msi::Column::build("MaxSize").nullable().int32(),
                msi::Column::build("MinDate").nullable().int32(),
                msi::Column::build("MaxDate").nullable().int32(),
                msi::Column::build("Languages").nullable().text_string(255),
            ],
        )
        .unwrap();

    // Upgrade
    package
        .create_table(
            "Upgrade",
            vec![
                msi::Column::build("UpgradeCode")
                    .primary_key()
                    .category(msi::Category::Guid)
                    .string(38),
                msi::Column::build("VersionMin")
                    .primary_key()
                    .nullable()
                    .text_string(20),
                msi::Column::build("VersionMax")
                    .primary_key()
                    .nullable()
                    .text_string(20),
                msi::Column::build("Language")
                    .primary_key()
                    .nullable()
                    .text_string(255),
                msi::Column::build("Attributes").primary_key().int32(),
                msi::Column::build("Remove").nullable().formatted_string(255),
                msi::Column::build("ActionProperty").id_string(72),
            ],
        )
        .unwrap();
    package.flush().unwrap();
}
