use crate::hb;
use std;
use std::ptr::NonNull;

use crate::common::{HarfbuzzObject, Language, Owned, Tag};

use std::fmt;
use std::io;
use std::io::Read;

pub type GlyphPosition = hb::hb_glyph_position_t;
pub type GlyphInfo = hb::hb_glyph_info_t;
pub type Feature = hb::hb_feature_t;

#[derive(Debug)]
pub(crate) struct GenericBuffer {
    raw: NonNull<hb::hb_buffer_t>,
}
impl GenericBuffer {
    pub(crate) fn new() -> Owned<GenericBuffer> {
        let buffer = unsafe { hb::hb_buffer_create() };
        unsafe { Owned::from_raw(buffer) }
    }

    pub(crate) fn len(&self) -> usize {
        unsafe { hb::hb_buffer_get_length(self.as_raw()) as usize }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn add_str(&mut self, string: &str) {
        let utf8_ptr = string.as_ptr() as *const i8;
        unsafe {
            hb::hb_buffer_add_utf8(
                self.as_raw(),
                utf8_ptr,
                string.len() as i32,
                0,
                string.len() as i32,
            );
        }
    }

    pub(crate) fn set_direction(&mut self, direction: hb::hb_direction_t) {
        unsafe { hb::hb_buffer_set_direction(self.as_raw(), direction) };
    }

    /// Returns the `Buffer`'s text direction.
    pub(crate) fn get_direction(&self) -> hb::hb_direction_t {
        unsafe { hb::hb_buffer_get_direction(self.as_raw()) }
    }

    pub(crate) fn set_language(&mut self, lang: Language) {
        unsafe { hb::hb_buffer_set_language(self.as_raw(), lang.0) }
    }

    pub(crate) fn get_language(&self) -> Option<Language> {
        let raw_lang = unsafe { hb::hb_buffer_get_language(self.as_raw()) };
        if raw_lang.is_null() {
            None
        } else {
            Some(Language(raw_lang))
        }
    }

    pub(crate) fn set_script(&mut self, script: hb::hb_script_t) {
        unsafe { hb::hb_buffer_set_script(self.as_raw(), script) }
    }

    pub(crate) fn get_script(&self) -> hb::hb_script_t {
        unsafe { hb::hb_buffer_get_script(self.as_raw()) }
    }

    pub(crate) fn guess_segment_properties(&mut self) {
        unsafe { hb::hb_buffer_guess_segment_properties(self.as_raw()) };
    }

    pub(crate) fn get_segment_properties(&self) -> hb::hb_segment_properties_t {
        unsafe {
            let mut segment_props: hb::hb_segment_properties_t = std::mem::uninitialized();
            hb::hb_buffer_get_segment_properties(self.as_raw(), &mut segment_props as *mut _);
            segment_props
        }
    }

    pub(crate) fn clear_contents(&mut self) {
        unsafe { hb::hb_buffer_clear_contents(self.as_raw()) };
    }

    pub(crate) fn get_glyph_positions(&self) -> &[GlyphPosition] {
        unsafe {
            let mut length: u32 = 0;
            let glyph_pos =
                hb::hb_buffer_get_glyph_positions(self.as_raw(), &mut length as *mut u32);
            std::slice::from_raw_parts(glyph_pos, length as usize)
        }
    }

    pub(crate) fn get_glyph_infos(&self) -> &[GlyphInfo] {
        unsafe {
            let mut length: u32 = 0;
            let glyph_infos = hb::hb_buffer_get_glyph_infos(self.as_raw(), &mut length as *mut u32);
            std::slice::from_raw_parts(glyph_infos, length as usize)
        }
    }

    /// Reverse the `Buffer`'s contents.
    pub(crate) fn reverse(&mut self) {
        unsafe { hb::hb_buffer_reverse(self.as_raw()) };
    }

    /// Reverse the `Buffer`'s contents in the range from `start` to `end`.
    pub(crate) fn reverse_range(&mut self, start: usize, end: usize) {
        assert!(start <= self.len(), end <= self.len());
        unsafe { hb::hb_buffer_reverse_range(self.as_raw(), start as u32, end as u32) }
    }

    pub(crate) fn content_type(&self) -> hb::hb_buffer_content_type_t {
        unsafe { hb::hb_buffer_get_content_type(self.as_raw()) }
    }
}

unsafe impl HarfbuzzObject for GenericBuffer {
    type Raw = hb::hb_buffer_t;

    unsafe fn from_raw(raw: *const Self::Raw) -> Self {
        GenericBuffer {
            raw: NonNull::new_unchecked(raw as *mut _),
        }
    }

    fn as_raw(&self) -> *mut Self::Raw {
        self.raw.as_ptr()
    }

    unsafe fn reference(&self) {
        hb::hb_buffer_reference(self.as_raw());
    }

    unsafe fn dereference(&self) {
        hb::hb_buffer_destroy(self.as_raw());
    }
}

/// The serialization format used in `BufferSerializer`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SerializeFormat {
    /// A human-readable, plain text format
    Text,
    /// A machine-readable JSON format.
    Json,
}

impl From<SerializeFormat> for hb::hb_buffer_serialize_format_t {
    fn from(fmt: SerializeFormat) -> Self {
        match fmt {
            SerializeFormat::Text => hb::HB_BUFFER_SERIALIZE_FORMAT_TEXT,
            SerializeFormat::Json => hb::HB_BUFFER_SERIALIZE_FORMAT_JSON,
        }
    }
}

bitflags! {
    /// Flags used for serialization with a `BufferSerializer`.
    #[derive(Default)]
    pub struct SerializeFlags: u32 {
        /// Do not serialize glyph cluster.
        const NO_CLUSTERS = hb::HB_BUFFER_SERIALIZE_FLAG_NO_CLUSTERS;
        /// Do not serialize glyph position information.
        const NO_POSITIONS = hb::HB_BUFFER_SERIALIZE_FLAG_NO_POSITIONS;
        /// Do no serialize glyph name.
        const NO_GLYPH_NAMES = hb::HB_BUFFER_SERIALIZE_FLAG_NO_GLYPH_NAMES;
        /// Serialize glyph extents.
        const GLYPH_EXTENTS = hb::HB_BUFFER_SERIALIZE_FLAG_GLYPH_EXTENTS;
        /// Serialize glyph flags.
        const GLYPH_FLAGS = hb::HB_BUFFER_SERIALIZE_FLAG_GLYPH_FLAGS;
        /// Do not serialize glyph advances, glyph offsets will reflect absolute
        /// glyph positions.
        const NO_ADVANCES = hb::HB_BUFFER_SERIALIZE_FLAG_NO_ADVANCES;
    }
}

/// A type that can be used to serialize a `GlyphBuffer`.
///
/// A `BufferSerializer` is obtained by calling the `GlyphBuffer::serializer`
/// method and provides a `Read` implementation that allows you to read the
/// serialized buffer contents.
#[derive(Debug)]
pub struct BufferSerializer<'a> {
    font: Option<&'a crate::Font<'a>>,
    buffer: &'a Owned<GenericBuffer>,
    start: usize,
    end: usize,
    format: SerializeFormat,
    flags: SerializeFlags,

    bytes: io::Cursor<Vec<u8>>,
}

impl<'a> Read for BufferSerializer<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.bytes.read(buf) {
            // if `bytes` is empty refill it
            Ok(0) => {
                if self.start >= self.end.saturating_sub(1) {
                    return Ok(0);
                }
                let mut bytes_written = 0;
                let num_serialized_items = unsafe {
                    hb::hb_buffer_serialize_glyphs(
                        self.buffer.as_raw(),
                        self.start as u32,
                        self.end as u32,
                        self.bytes.get_mut().as_mut_ptr() as *mut _,
                        self.bytes.get_ref().capacity() as u32,
                        &mut bytes_written,
                        self.font
                            .map(|f| f.as_raw())
                            .unwrap_or(std::ptr::null_mut()),
                        self.format.into(),
                        self.flags.bits(),
                    )
                };
                self.start += num_serialized_items as usize;
                self.bytes.set_position(0);
                unsafe { self.bytes.get_mut().set_len(bytes_written as usize) };

                self.read(buf)
            }
            Ok(size) => Ok(size),
            Err(err) => Err(err),
        }
    }
}

/// This type provides an interface to create one of the buffer types from a raw harfbuzz pointer.
#[derive(Debug)]
pub enum TypedBuffer {
    /// Contains a `UnicodeBuffer`
    Unicode(UnicodeBuffer),
    /// Contains a `GlyphBuffer`
    Glyphs(GlyphBuffer),
}

impl TypedBuffer {
    /// Takes ownership of the raw `hb_buffer_t` object and converts it to are
    /// `TypedBuffer`. If no safe conversion is possible returns `None`.
    pub unsafe fn take_from_raw(raw: *mut hb::hb_buffer_t) -> Option<TypedBuffer> {
        let generic_buf: Owned<GenericBuffer> = Owned::from_raw(raw);
        let content_type = generic_buf.content_type();
        match content_type {
            hb::HB_BUFFER_CONTENT_TYPE_UNICODE => {
                Some(TypedBuffer::Unicode(UnicodeBuffer(generic_buf)))
            }
            hb::HB_BUFFER_CONTENT_TYPE_GLYPHS => {
                Some(TypedBuffer::Glyphs(GlyphBuffer(generic_buf)))
            }
            _ => None,
        }
    }
}

/// A `UnicodeBuffer` can be filled with unicode text and corresponding cluster
/// indices.
///
/// # Usage
///
/// The buffer manages an allocation for the unicode codepoints to be shaped.
/// This allocation is reused for storing the results of the shaping operation
/// in a `GlyphBuffer` object. The intended usage is to keep one (or e.g. one
/// per thread) `UnicodeBuffer` around. When needed, you fill it with text that
/// should be shaped and pass it as an argument to the `shape` function. That
/// method returns a `GlyphBuffer` object containing the shaped glyph indices.
/// Once you got the needed information out of the `GlyphBuffer` you call its
/// `.clear()` method which in turn gives you a fresh `UnicodeBuffer` (also
/// reusing the original allocation again). This buffer can then be used to
/// shape more text.
///
/// # Interaction with the raw harfbuzz API
///
/// If you want to get a `UnicodeBuffer` from a pointer to a raw harfbuzz
/// object, you need to use the `from_raw` static method on `TypedBuffer`. This
/// ensures that a buffer of correct type is created.
pub struct UnicodeBuffer(pub(crate) Owned<GenericBuffer>);
impl UnicodeBuffer {
    /// Creates a new empty `Buffer`.
    ///
    /// # Examples
    /// ```
    /// use harfbuzz_rs::UnicodeBuffer;
    ///
    /// let buffer = UnicodeBuffer::new();
    /// assert!(buffer.is_empty());
    /// ```
    pub fn new() -> UnicodeBuffer {
        UnicodeBuffer(GenericBuffer::new())
    }

    /// Converts this buffer to a raw harfbuzz object pointer.
    pub fn into_raw(self) -> *mut hb::hb_buffer_t {
        Owned::into_raw(self.0)
    }

    /// Returns the length of the data of the buffer.
    ///
    /// This corresponds to the number of unicode codepoints contained in the
    /// buffer.
    ///
    /// # Examples
    /// ```
    /// use harfbuzz_rs::UnicodeBuffer;
    ///
    /// let str1 = "Hello ";
    /// let buffer = UnicodeBuffer::new().add_str(str1);
    /// assert_eq!(buffer.len(), str1.len());
    ///
    /// let str2 = "😍🙈";
    /// let buffer = buffer.add_str(str2);
    /// assert_eq!(buffer.len(), str1.len() + 2);;
    /// ```
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the buffer contains no elements.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Add the string slice `str_slice` to the `Buffer`'s array of codepoints.
    ///
    /// # Examples
    /// ```
    /// use harfbuzz_rs::UnicodeBuffer;
    ///
    /// let buffer = UnicodeBuffer::new().add_str("Hello");
    /// let buffer = buffer.add_str(" World");
    /// assert_eq!(buffer.string_lossy(), "Hello World");
    /// ```
    pub fn add_str(mut self, str_slice: &str) -> UnicodeBuffer {
        self.0.add_str(str_slice);
        self
    }

    /// Returns an Iterator over the stored unicode codepoints.
    ///
    /// # Examples
    /// ```
    /// use harfbuzz_rs::UnicodeBuffer;
    ///
    /// let buffer = UnicodeBuffer::new().add_str("ab");
    /// let mut iterator = buffer.codepoints();
    ///
    /// assert_eq!('a' as u32, iterator.next().unwrap());
    /// assert_eq!('b' as u32, iterator.next().unwrap());
    /// assert!(iterator.next().is_none());
    /// ```
    pub fn codepoints(&self) -> Codepoints<'_> {
        Codepoints {
            slice_iter: self.0.get_glyph_infos().iter(),
        }
    }

    /// Get the stored codepoints as a `String`.
    ///
    /// Invalid codepoints get replaced by the U+FFFD replacement character.
    pub fn string_lossy(&self) -> String {
        self.codepoints()
            .map(|cp| std::char::from_u32(cp).unwrap_or('\u{FFFD}'))
            .collect()
    }

    /// Set the text direction of the `Buffer`'s contents.
    pub fn set_direction(mut self, direction: hb::hb_direction_t) -> UnicodeBuffer {
        self.0.set_direction(direction);
        self
    }

    /// Returns the `Buffer`'s text direction.
    pub fn get_direction(&self) -> hb::hb_direction_t {
        self.0.get_direction()
    }

    /// Set the script from an ISO15924 tag.
    pub fn set_script(mut self, script: Tag) -> UnicodeBuffer {
        self.0
            .set_script(unsafe { hb::hb_script_from_iso15924_tag(script.0) });
        self
    }

    /// Get the ISO15924 script tag.
    pub fn get_script(&self) -> Tag {
        Tag(unsafe { hb::hb_script_to_iso15924_tag(self.0.get_script()) })
    }

    /// Set the buffer language.
    pub fn set_language(mut self, lang: Language) -> UnicodeBuffer {
        self.0.set_language(lang);
        self
    }

    /// Get the buffer language.
    pub fn get_language(&self) -> Option<Language> {
        self.0.get_language()
    }

    /// Guess the segment properties (direction, language, script) for the
    /// current buffer.
    pub fn guess_segment_properties(mut self) -> UnicodeBuffer {
        self.0.guess_segment_properties();
        self
    }

    /// Get the segment properties (direction, language, script) of the current
    /// buffer.
    pub fn get_segment_properties(&self) -> hb::hb_segment_properties_t {
        self.0.get_segment_properties()
    }

    /// Clear the contents of the buffer (i.e. the stored string of unicode
    /// characters).
    ///
    /// # Examples
    /// ```
    /// use harfbuzz_rs::UnicodeBuffer;
    ///
    /// let buffer = UnicodeBuffer::new();
    /// let buffer = buffer.add_str("Test!");
    /// assert_eq!(buffer.len(), 5);
    /// let buffer = buffer.clear_contents();
    /// assert!(buffer.is_empty());
    /// ```
    pub fn clear_contents(mut self) -> UnicodeBuffer {
        self.0.clear_contents();
        self
    }
}

impl std::fmt::Debug for UnicodeBuffer {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("UnicodeBuffer")
            .field("content", &self.string_lossy())
            .field("direction", &self.get_direction())
            .field("language", &self.get_language())
            .field("script", &self.get_script())
            .finish()
    }
}

impl std::default::Default for UnicodeBuffer {
    fn default() -> UnicodeBuffer {
        UnicodeBuffer::new()
    }
}

/// An iterator over the codepoints stored in a `UnicodeBuffer`.
///
/// You get an iterator of this type from the `.codepoints()` method on
/// `UnicodeBuffer`. I yields `u32`s that should be interpreted as unicode
/// codepoints stored in the underlying buffer.
#[derive(Debug, Clone)]
pub struct Codepoints<'a> {
    slice_iter: std::slice::Iter<'a, GlyphInfo>,
}

impl<'a> Iterator for Codepoints<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        self.slice_iter.next().map(|info| info.codepoint)
    }
}

/// A `GlyphBuffer` contains the resulting output information of the shaping
/// process.
///
/// An object of this type is obtained through the `shape` function.
pub struct GlyphBuffer(pub(crate) Owned<GenericBuffer>);

impl GlyphBuffer {
    /// Returns the length of the data of the buffer.
    ///
    /// When called before shaping this is the number of unicode codepoints
    /// contained in the buffer. When called after shaping it returns the number
    /// of glyphs stored.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Converts this buffer to a raw harfbuzz object pointer.
    pub fn into_raw(self) -> *mut hb::hb_buffer_t {
        Owned::into_raw(self.0)
    }

    /// Returns `true` if the buffer contains no elements.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the glyph positions.
    pub fn get_glyph_positions(&self) -> &[GlyphPosition] {
        self.0.get_glyph_positions()
    }

    /// Get the glyph infos.
    pub fn get_glyph_infos(&self) -> &[GlyphInfo] {
        self.0.get_glyph_infos()
    }

    /// Reverse the `Buffer`'s contents.
    pub fn reverse(&mut self) {
        self.0.reverse()
    }

    /// Reverse the `Buffer`'s contents in the range from `start` to `end`.
    pub fn reverse_range(&mut self, start: usize, end: usize) {
        self.0.reverse_range(start, end)
    }

    /// Clears the contents of the glyph buffer and returns an empty
    /// `UnicodeBuffer` reusing the existing allocation.
    pub fn clear(mut self) -> UnicodeBuffer {
        self.0.clear_contents();
        UnicodeBuffer(self.0)
    }

    /// Returns a serializer that allows the contents of the buffer to be
    /// converted into a human or machine readable representation.
    ///
    /// # Arguments
    /// - `font`: Optionally a font can be provided for access to glyph names
    ///   and glyph extents. If `None` is passed an empty font is assumed.
    /// - `format`: The serialization format to use.
    /// - `flags`: Allows you to control which information will be contained in
    ///   the serialized output.
    ///
    /// # Examples
    ///
    /// Serialize the glyph buffer contents to a string using the textual format
    /// without any special flags.
    /// ```
    /// use harfbuzz_rs::*;
    /// use std::io::Read;
    /// # use std::path::PathBuf;
    /// # let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    /// # path.push("testfiles/SourceSansVariable-Roman.ttf");
    /// let face = Face::from_file(path, 0).expect("Error reading font file.");
    /// let font = Font::new(face);
    ///
    /// let buffer = UnicodeBuffer::new().add_str("ABC");
    ///
    /// let buffer = shape(&font, buffer, &[]);
    ///
    /// let mut string = String::new();
    /// buffer
    ///     .serializer(
    ///         Some(&font),
    ///         SerializeFormat::Text,
    ///         SerializeFlags::default(),
    ///     ).read_to_string(&mut string)
    ///     .unwrap();
    ///
    /// assert_eq!(string, "gid2=0+520|gid3=1+574|gid4=2+562")
    /// ```
    pub fn serializer<'a>(
        &'a self,
        font: Option<&'a crate::Font<'a>>,
        format: SerializeFormat,
        flags: SerializeFlags,
    ) -> BufferSerializer<'a> {
        BufferSerializer {
            font,
            buffer: &self.0,
            start: 0,
            end: self.len(),
            format,
            flags,
            bytes: io::Cursor::new(Vec::with_capacity(128)),
        }
    }
}

impl fmt::Debug for GlyphBuffer {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("GlyphBuffer")
            .field("glyph_positions", &self.get_glyph_positions())
            .field("glyph_infos", &self.get_glyph_infos())
            .finish()
    }
}

impl fmt::Display for GlyphBuffer {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut serializer =
            self.serializer(None, SerializeFormat::Text, SerializeFlags::default());
        let mut string = String::new();
        serializer.read_to_string(&mut string).unwrap();
        write!(fmt, "{}", string)?;
        Ok(())
    }
}
