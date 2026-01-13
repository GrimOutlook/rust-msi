use crate::internal::codepage::CodePage;
use crate::internal::language::Language;
use crate::internal::propset::{OperatingSystem, PropertySet, PropertyValue};
use crate::internal::timestamp::Timestamp;
use std::io::{self, Read, Seek, Write};
use std::time::SystemTime;
use time::UtcDateTime;
use uuid::Uuid;

// ========================================================================= //

// This constant is this UUID:
//     F29F85E0-4FF9-1068-AB91-08002B27B3D9
// Which comes from this page:
//     https://msdn.microsoft.com/en-us/library/windows/desktop/
//     aa380052(v=vs.85).aspx
// The first three fields are in little-endian, and the last two in big-endian,
// because that's how Windows encodes UUIDs.  For details, see:
//     https://en.wikipedia.org/wiki/Universally_unique_identifier#Encoding
const FMTID: [u8; 16] =
    *b"\xe0\x85\x9f\xf2\xf9\x4f\x68\x10\xab\x91\x08\x00\x2b\x27\xb3\xd9";

#[repr(u32)]
pub enum SummaryInfoField {
    Codepage = 1,
    Title,
    Subject,
    Author,
    Keywords,
    Comments,
    Template,
    LastSavedBy,
    RevisionNumber,
    LastPrinted,
    CreateDateTime,
    LastSaveDateTime,
    PageCount,
    WordCount,
    CharacterCount,
    CreatingApplication,
    Security,
}

pub enum Architecture {
    X64,
    Intel,
    Intel64,
    Arm,
    Arm64,
}

/// [Documentation](https://learn.microsoft.com/en-us/windows/win32/msi/template-summary)
pub enum Template {
    /// For an installation package, the Template Summary property indicates the
    /// platform and language versions that are compatible with this
    /// installation database.
    Installation { arch: Option<Architecture>, languages: Vec<Language> },
    /// The Template Summary property of a transform indicates the platform and
    /// language versions compatible with the transform. In a transform file,
    /// only one language may be specified.
    Transform { arch: Option<Architecture>, language: Option<Language> },
    /// The Template Summary property of a patch package is a
    /// semicolon-delimited list of the product codes that can accept the patch.
    Patch { product_codes: Vec<Uuid> },
}

pub enum WindowsInstallerVersion {
    V2_0 = 200,
    V3_0 = 300,
    V3_1 = 301,
    V4_5 = 405,
    V5_0 = 500,
}

pub enum FilenameLength {
    /// MSI uses long filenames
    LongFilenames,
    /// MSI uses short filenames
    ShortFilenames,
}

pub enum SourceCompression {
    /// Source is uncompressed.
    Uncompressed,
    /// Source is compressed.
    Compressed,
}

pub enum SourceMedia {
    /// Source is original media.
    Original,
    /// Source is an administrative image created by an administrative
    /// installation.
    AdminImage,
}

pub enum ElevatedPrivileges {
    Required,
    NotRequired,
}

pub struct WordCount {
    filename_length: FilenameLength,
    compressions: SourceCompression,
    source: SourceMedia,
    elevated_privileges: ElevatedPrivileges,
}

// ========================================================================= //

/// Summary information (e.g. title, author) about an MSI package.
pub struct SummaryInfo {
    codepage: CodePage,
    title: String,
    subject: String,
    author: String,
    keywords: Vec<String>,
    comments: String,
    template: Template,
    last_saved_by: String,
    revision_number: Uuid,
    last_printed: UtcDateTime,
    created_datetime: UtcDateTime,
    last_saved_datetime: UtcDateTime,
    page_count: WindowsInstallerVersion,
    word_count: WordCount,
}

impl SummaryInfo {
    /// Creates an empty `SummaryInfo` with no properties set.
    pub(crate) fn new() -> SummaryInfo {
        let properties = PropertySet::new(OperatingSystem::Win32, 10, FMTID);
        let mut summary = SummaryInfo { properties };
        summary.set_codepage(CodePage::Utf8);
        summary
    }

    pub(crate) fn read<R: Read + Seek>(reader: R) -> io::Result<SummaryInfo> {
        let properties = PropertySet::read(reader)?;
        if properties.format_identifier() != &FMTID {
            invalid_data!("Property set has wrong format identifier");
        }
        Ok(SummaryInfo { properties })
    }

    pub(crate) fn write<W: Write>(&self, writer: W) -> io::Result<()> {
        Into::<PropertySet>::into(self).write(writer)
    }
}

impl Into<PropertySet> for SummaryInfo {
    fn into(self) -> PropertySet {
        todo!()
    }
}

// ========================================================================= //

#[cfg(test)]
mod tests {
    use super::SummaryInfo;
    use crate::internal::language::Language;
    use std::time::{Duration, UNIX_EPOCH};
    use uuid::Uuid;

    #[test]
    fn set_properties() {
        let languages = vec![
            Language::from_tag("en-CA"),
            Language::from_tag("fr-CA"),
            Language::from_tag("en-US"),
            Language::from_tag("es-MX"),
        ];
        let timestamp = UNIX_EPOCH + Duration::from_secs(12345678);
        let uuid =
            Uuid::parse_str("0000002a-000c-0005-0c03-0938362b0809").unwrap();

        let mut summary_info = SummaryInfo::new();
        summary_info.set_arch("x64");
        summary_info.set_author("Jane Doe");
        summary_info.set_comments("This app is the greatest!");
        summary_info.set_creating_application("cargo-test");
        summary_info.set_creation_time(timestamp);
        summary_info.set_languages(&languages);
        summary_info.set_subject("My Great App");
        summary_info.set_title("Installation Package");
        summary_info.set_uuid(uuid);
        summary_info.set_word_count(2);

        assert_eq!(summary_info.arch(), Some("x64"));
        assert_eq!(summary_info.author(), Some("Jane Doe"));
        assert_eq!(summary_info.comments(), Some("This app is the greatest!"));
        assert_eq!(summary_info.creating_application(), Some("cargo-test"));
        assert_eq!(summary_info.creation_time(), Some(timestamp));
        assert_eq!(summary_info.languages(), languages);
        assert_eq!(summary_info.subject(), Some("My Great App"));
        assert_eq!(summary_info.title(), Some("Installation Package"));
        assert_eq!(summary_info.uuid(), Some(uuid));
        assert_eq!(summary_info.word_count(), Some(2));

        summary_info.clear_arch();
        assert_eq!(summary_info.arch(), None);
        summary_info.clear_author();
        assert_eq!(summary_info.author(), None);
        summary_info.clear_comments();
        assert_eq!(summary_info.comments(), None);
        summary_info.clear_creating_application();
        assert_eq!(summary_info.creating_application(), None);
        summary_info.clear_creation_time();
        assert_eq!(summary_info.creation_time(), None);
        summary_info.clear_languages();
        assert_eq!(summary_info.languages(), Vec::new());
        summary_info.clear_subject();
        assert_eq!(summary_info.subject(), None);
        summary_info.clear_title();
        assert_eq!(summary_info.title(), None);
        summary_info.clear_uuid();
        assert_eq!(summary_info.uuid(), None);
        summary_info.clear_word_count();
        assert_eq!(summary_info.word_count(), None);
    }

    #[test]
    fn template_property() {
        // Set language before setting arch:
        let mut summary_info = SummaryInfo::new();
        assert_eq!(summary_info.arch(), None);
        summary_info.set_languages(&[Language::from_tag("en")]);
        assert_eq!(summary_info.arch(), None);
        assert_eq!(summary_info.languages(), vec![Language::from_tag("en")]);
        summary_info.set_arch("Intel");
        assert_eq!(summary_info.arch(), Some("Intel"));
        assert_eq!(summary_info.languages(), vec![Language::from_tag("en")]);

        // Set arch before setting language:
        let mut summary_info = SummaryInfo::new();
        assert_eq!(summary_info.languages(), vec![]);
        assert_eq!(summary_info.arch(), None);
        summary_info.set_arch("Intel");
        assert_eq!(summary_info.languages(), vec![]);
        assert_eq!(summary_info.arch(), Some("Intel"));
        summary_info.set_languages(&[Language::from_tag("en")]);
        assert_eq!(summary_info.languages(), vec![Language::from_tag("en")]);
        assert_eq!(summary_info.arch(), Some("Intel"));
    }
}

// ========================================================================= //
