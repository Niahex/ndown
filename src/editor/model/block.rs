use std::io::{self, Write};

#[derive(Clone, Debug, PartialEq)]
pub enum BlockType {
    Paragraph,
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Quote,
    CodeBlock,
}

#[derive(Clone, Debug, Copy, Default)]
pub struct StyleBits {
    pub is_bold: bool,
    pub is_italic: bool,
    pub is_code: bool,
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
}

#[derive(Clone, Debug)]
pub struct Block {
    pub id: u64,
    pub ty: BlockType,
    pub text: String,
    pub styles: Vec<StyleSpan>,
    pub layout_cache: Option<BlockLayoutCache>,
    pub is_dirty: bool,
}

impl Block {
    pub fn new(id: u64, ty: BlockType, text: &str) -> Self {
        Self {
            id,
            ty,
            text: text.to_string(),
            styles: vec![StyleSpan {
                len: text.chars().count(),
                style: StyleBits::default(),
            }],
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
            if span.style.is_code {
                buf.push('`');
            }
            if span.style.is_bold {
                buf.push_str("**");
            }
            if span.style.is_italic {
                buf.push('*');
            }
            for _ in 0..span.len {
                if let Some(c) = char_iter.next() {
                    buf.push(c);
                }
            }
            if span.style.is_italic {
                buf.push('*');
            }
            if span.style.is_bold {
                buf.push_str("**");
            }
            if span.style.is_code {
                buf.push('`');
            }
        }
    }

    // Version Streaming I/O (Zero allocation for big strings)
    pub fn write_markdown_to_writer<W: Write>(&self, w: &mut W) -> io::Result<()> {
        let mut char_iter = self.text.chars();
        // Buffer local pour éviter trop d'appels à write (optionnel si BufWriter est utilisé au dessus)
        // Mais BufWriter est byte-based, ici on manipule des chars/strings.
        // On écrit direct les slices.

        for span in &self.styles {
            if span.style.is_code {
                w.write_all(b"`")?;
            }
            if span.style.is_bold {
                w.write_all(b"**")?;
            }
            if span.style.is_italic {
                w.write_all(b"*")?;
            }

            // Pour le texte, on ne peut pas faire write_all direct car on doit découper par span
            // et text est utf8.
            // On doit extraire la sous-chaîne correspondante.
            // Pas de zero-copy facile ici sans indices de bytes.
            // On va reconstruire une petite string temporaire ou écrire char par char.
            // Écrire char par char dans un BufWriter est très efficace.

            // Option plus rapide : trouver les byte indices et écrire le slice.
            // Comme on itère séquentiellement, on peut garder un byte offset.
            // C'est ce qu'on va faire pour l'optimisation ultime.

            // MAIS wait, char_iter consomme. On ne peut pas facilement mapper char -> byte index sans re-parcourir.
            // On va écrire char par char dans un petit buffer stack (encode_utf8).

            let mut b = [0; 4]; // Max utf8 char len
            for _ in 0..span.len {
                if let Some(c) = char_iter.next() {
                    let s = c.encode_utf8(&mut b);
                    w.write_all(s.as_bytes())?;
                }
            }

            if span.style.is_italic {
                w.write_all(b"*")?;
            }
            if span.style.is_bold {
                w.write_all(b"**")?;
            }
            if span.style.is_code {
                w.write_all(b"`")?;
            }
        }
        Ok(())
    }

    pub fn to_markdown(&self) -> String {
        let mut s = String::with_capacity(self.text.len() + 10);
        self.write_markdown_to(&mut s);
        s
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
        self.layout_cache = None;
    }
}
