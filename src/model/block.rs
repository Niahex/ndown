#[derive(Clone, Debug, PartialEq)]
pub enum BlockType {
    Paragraph,
    Heading1,
    Heading2,
    Heading3,
    Quote,
    CodeBlock,
}

#[derive(Clone, Debug)]
pub struct TextSpan {
    pub text: String,
    pub is_bold: bool,
    pub is_italic: bool,
    pub is_code: bool,
}

impl TextSpan {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            is_bold: false,
            is_italic: false,
            is_code: false,
        }
    }
    
    pub fn len(&self) -> usize {
        self.text.chars().count()
    }
}

#[derive(Clone, Debug)]
pub struct Block {
    pub id: u64,
    pub ty: BlockType,
    pub content: Vec<TextSpan>,
    
    // Cache
    pub height: f64,
    pub is_dirty: bool,
}

impl Block {
    pub fn new(id: u64, ty: BlockType, text: &str) -> Self {
        Self {
            id,
            ty,
            content: vec![TextSpan::new(text)],
            height: 0.0,
            is_dirty: true,
        }
    }
    
    pub fn text_len(&self) -> usize {
        self.content.iter().map(|s| s.len()).sum()
    }
    
    pub fn full_text(&self) -> String {
        self.content.iter().map(|s| s.text.clone()).collect()
    }

    // Version optimisée qui écrit dans un buffer existant
    pub fn write_markdown_to(&self, buf: &mut String) {
        for span in &self.content {
            if span.is_code { buf.push('`'); }
            if span.is_bold { buf.push_str("**"); }
            if span.is_italic { buf.push('*'); }
            
            buf.push_str(&span.text);
            
            if span.is_italic { buf.push('*'); }
            if span.is_bold { buf.push_str("**"); }
            if span.is_code { buf.push('`'); }
        }
    }

    // Wrapper de compatibilité (mais qui alloue)
    pub fn to_markdown(&self) -> String {
        let mut s = String::with_capacity(self.text_len() + 10);
        self.write_markdown_to(&mut s);
        s
    }
    
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }
}
