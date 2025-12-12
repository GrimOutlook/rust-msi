#!/usr/bin/env bash

# Color information found here https://misc.flogisoft.com/bash/tip_colors_and_formatting
function info() {
  echo -e "\e[36m   $*\e[0m" >&2
}

function success() {
  echo -e "\e[32m   $*\e[0m" >&2
}

function warn() {
  echo -e "\e[33m   $*\e[0m" >&2
}

function fail() {
  echo -e "\e[31m   $*\e[0m" >&2
}

function fatal() {
  fail "$*"
  exit 1
}

WIXL_MSI="WIXL_MSI.msi"
DUPLICATE_MSI="DUPLICATE_MSI.msi"

# Remove the old files
rm $WIXL_MSI $DUPLICATE_MSI

# Build the binary separately so the MSIs are built as close together as
# possible temporally.
cargo build --example duplicate || fatal "Failed to build \`duplicate\` example"

wixl Test.wxs -o $WIXL_MSI || fatal "Failed to build \`Test.msi\` using \`msitools\`"

REVISION_NUMBER="$(msiinfo suminfo $WIXL_MSI | grep -oP "Revision number \(UUID\): \K.*")"

# Generate the duplicate MSI.
./target/debug/examples/duplicate --revision-number "$REVISION_NUMBER" $DUPLICATE_MSI

# Check for differences in the Summary Header
WIXL_SUMINFO=$(msiinfo suminfo $WIXL_MSI)
DUPLICATE_SUMINFO=$(msiinfo suminfo $DUPLICATE_MSI)

if test "$WIXL_SUMINFO" == "$DUPLICATE_SUMINFO"; then
  success "SummaryInformation is the same"
else
  fail "SummaryInformation contains differences"
  difft <($WIXL_SUMINFO) <($DUPLICATE_SUMINFO)
fi

# Check that the list of tables is the same
WIXL_TABLES=$(msiinfo tables $WIXL_MSI | sort)
DUPLICATE_TABLES=$(msiinfo tables $DUPLICATE_MSI | sort)
warn "Cannot create tables in a different order using current \`rust-msi\` implementation."
warn "Only a table's existence in list is currently tested."

if test "$WIXL_TABLES" == "$DUPLICATE_TABLES"; then
  success "Tables list is the same"
else
  fail "List of tables is different"
  difft <(echo "$WIXL_TABLES") <(echo "$DUPLICATE_TABLES")
  fatal "Cannot continue if tables list is different"
fi

# Check that the list of streams is the same
WIXL_STREAMS=$(msiinfo streams $WIXL_MSI | sort)
DUPLICATE_STREAMS=$(msiinfo streams $DUPLICATE_MSI | sort)

if test "$WIXL_STREAMS" == "$DUPLICATE_STREAMS"; then
  success "Streams list is the same"
else
  fail "List of streams is different"
  difft <(echo "$WIXL_STREAMS") <(echo "$DUPLICATE_STREAMS")
fi

# Dump all of the tables so we can compare the .idt files
function dump_msi() {
  MSI=$1
  DUMP_DIR=$(mktemp -d)
  msidump -t -d "$DUMP_DIR" "$MSI" >/dev/null || fatal "Failed to dump $MSI table data to $DUMP_DIR"
  info "Dumped $MSI tables to $DUMP_DIR"
  echo "$DUMP_DIR"
}
WIXL_DUMP_DIR=$(dump_msi $WIXL_MSI)
DUPLICATE_DUMP_DIR=$(dump_msi $DUPLICATE_MSI)
shopt -s nullglob
for table_file in "$WIXL_DUMP_DIR"/*; do
  table=$(basename "$table_file")
  WIXL_TABLE_FILE="$WIXL_DUMP_DIR/$table"
  DUPLICATE_TABLE_FILE="$DUPLICATE_DUMP_DIR/$table"
  if diff "$WIXL_TABLE_FILE" "$DUPLICATE_TABLE_FILE" >/dev/null; then
    success "$table is the same for both files"
  else
    fail "$table has differences"
    difft "$WIXL_TABLE_FILE" "$DUPLICATE_TABLE_FILE"
  fi
done
