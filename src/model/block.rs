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

// Un intervalle de style. Les intervalles sont stockés à plat et contigus pour le rendu.
#[derive(Clone, Debug)]
pub struct StyleSpan {
    pub len: usize, // Longueur en caractères
    pub style: StyleBits,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub id: u64,
    pub ty: BlockType,
    pub text: String, // Le texte brut visible (sans marqueurs Markdown)
    pub styles: Vec<StyleSpan>, // La liste des styles appliqués séquentiellement
    
    pub height: f64,
    pub is_dirty: bool,
}

impl Block {
    pub fn new(id: u64, ty: BlockType, text: &str) -> Self {
        Self {
            id,
            ty,
            text: text.to_string(),
            styles: vec![StyleSpan { len: text.chars().count(), style: StyleBits::default() }],
            height: 0.0,
            is_dirty: true,
        }
    }
    
    pub fn text_len(&self) -> usize {
        self.text.chars().count()
    }
    
    pub fn full_text(&self) -> &str {
        &self.text
    }

    // Reconstruit le markdown en injectant les marqueurs aux frontières des styles
    pub fn write_markdown_to(&self, buf: &mut String) {
        let mut char_iter = self.text.chars();
        
        for span in &self.styles {
            // Ouvrir les tags
            if span.style.is_code { buf.push('`'); }
            if span.style.is_bold { buf.push_str("**"); }
            if span.style.is_italic { buf.push('*'); }
            
            // Écrire le texte du span
            for _ in 0..span.len {
                if let Some(c) = char_iter.next() {
                    buf.push(c);
                }
            }
            
            // Fermer les tags (ordre inverse idéalement, mais simple ici pour MD)
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
    }
}