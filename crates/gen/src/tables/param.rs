use super::*;
macros::table!(Param);

impl Param {
    pub fn flags(&self) -> ParamFlags {
        ParamFlags(self.reader.u32(self.row, 0))
    }

    pub fn sequence(&self) -> u32 {
        self.reader.u32(self.row, 1)
    }

    pub fn name(&self) -> &'static str {
        self.reader.str(self.row, 2)
    }

    pub fn attributes(&self) -> impl Iterator<Item = Attribute> + '_ {
        self.reader
            .equal_range(
                self.row.file_index,
                TableIndex::CustomAttribute,
                0,
                HasAttribute::Param(*self).encode(),
            )
            .map(move |row| Attribute {
                reader: self.reader,
                row,
            })
    }

    pub fn has_attribute(&self, namespace: &str, name: &str) -> bool {
        self.attributes()
            .any(|attribute| attribute.full_name() == (namespace, name))
    }

    pub fn is_input(&self) -> bool {
        // TODO: workaround for https://github.com/microsoft/win32metadata/issues/63
        let mut is_input = !self.flags().output();

        if is_input && self.has_attribute("Windows.Win32.Interop", "ComOutPtrAttribute") {
            is_input = false;
        }

        is_input
    }
}
