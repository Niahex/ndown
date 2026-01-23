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
    pub height_cache: f64,
}

impl Block {
    pub fn new(id: u64, ty: BlockType, text: &str) -> Self {
        Self {
            id,
            ty,
            content: vec![TextSpan::new(text)],
            height_cache: 0.0,
        }
    }
    
    pub fn text_len(&self) -> usize {
        self.content.iter().map(|s| s.len()).sum()
    }
    
    // Retourne le texte pur (pour l'affichage curseur, recherche simple)
    pub fn full_text(&self) -> String {
        self.content.iter().map(|s| s.text.clone()).collect()
    }

    // Reconstruit le markdown source (pour le re-parsing)
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        for span in &self.content {
            if span.is_code { md.push('`'); }
            if span.is_bold { md.push_str("**"); }
            if span.is_italic { md.push('*'); }
            
            md.push_str(&span.text);
            
            if span.is_italic { md.push('*'); }
            if span.is_bold { md.push_str("**"); }
            if span.is_code { md.push('`'); }
        }
        md
    }
}
