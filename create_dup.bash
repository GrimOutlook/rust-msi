#!/usr/bin/env bash

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

bash diff_msi.bash
