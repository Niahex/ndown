#[derive(Clone, Debug, PartialEq)]
pub enum BlockType {
    Paragraph,
    Heading1,
    Heading2,
    Heading3,
    Quote,
    CodeBlock,
}

#[derive(Clone, Debug, Copy)]
pub struct StyleBits {
    pub is_bold: bool,
    pub is_italic: bool,
    pub is_code: bool,
}

impl Default for StyleBits {
    fn default() -> Self {
        Self { is_bold: false, is_italic: false, is_code: false }
    }
}

#[derive(Clone, Debug)]
pub struct StyleSpan {
    pub len: usize,
    pub style: StyleBits,
}

#[derive(Clone, Debug)]
pub struct BlockLayoutCache {
    pub height: f64,
    pub width: f64,
    // On pourrait stocker la position des glyphes ici pour le hit testing ultra-rapide
    // Mais pour l'instant, height suffit pour le scrolling
}

#[derive(Clone, Debug)]
pub struct Block {
    pub id: u64,
    pub ty: BlockType,
    pub text: String,
    pub styles: Vec<StyleSpan>,
    
    // Cache
    pub layout_cache: Option<BlockLayoutCache>, // Nouveau cache
    pub is_dirty: bool,
}

impl Block {
    pub fn new(id: u64, ty: BlockType, text: &str) -> Self {
        Self {
            id,
            ty,
            text: text.to_string(),
            styles: vec![StyleSpan { len: text.chars().count(), style: StyleBits::default() }],
            layout_cache: None,
            is_dirty: true,
        }
    }
    
    pub fn text_len(&self) -> usize {
        self.text.chars().count()
    }
    
    pub fn full_text(&self) -> &str {
        &self.text
    }

    pub fn write_markdown_to(&self, buf: &mut String) {
        let mut char_iter = self.text.chars();
        
        for span in &self.styles {
            if span.style.is_code { buf.push('`'); }
            if span.style.is_bold { buf.push_str("**"); }
            if span.style.is_italic { buf.push('*'); }
            
            for _ in 0..span.len {
                if let Some(c) = char_iter.next() {
                    buf.push(c);
                }
            }
            
            if span.style.is_italic { buf.push('*'); }
            if span.style.is_bold { buf.push_str("**"); }
            if span.style.is_code { buf.push('`'); }
        }
    }

    pub fn to_markdown(&self) -> String {
        let mut s = String::with_capacity(self.text.len() + 10);
        self.write_markdown_to(&mut s);
        s
    }
    
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
        self.layout_cache = None; // Invalider le cache
    }
}
