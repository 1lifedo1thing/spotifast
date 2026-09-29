//! Colour emoji in names, titles and lyrics, in the platform's own style.
//!
//! fastframe-emoji finds the platform's colour emoji font (Apple Color Emoji,
//! Segoe UI Emoji, the desktop's font on Linux) and draws each emoji cluster
//! as a picture on a worker thread. Spotifast lays its text out exactly as
//! before, with the bundled monochrome face standing in for each emoji, then
//! makes those glyphs transparent and paints the colour picture over them.
//! Layout, truncation and right-to-left reordering therefore never change,
//! and text without an emoji takes the plain path untouched.
//!
//! Without a colour emoji font (a Linux desktop with none installed) nothing
//! is hidden: the monochrome face draws the emoji, as it always has.

use std::ops::Range;
use std::sync::Arc;

use egui::{Color32, Galley, Painter, Pos2, Rect, Ui};

/// Chooses the platform's emoji font and starts finding it off this thread.
/// `synchronous` draws each picture in the frame that first shows it, for
/// demo captures that must show every emoji in their first frame.
pub fn install(synchronous: bool) {
    fastframe_emoji::EmojiSetup::default()
        .system(true)
        .synchronous(synchronous)
        .install();
    std::thread::spawn(fastframe_emoji::warm_up);
}

/// Whether `text` holds an emoji this machine can draw in colour.
pub fn shows(text: &str) -> bool {
    fastframe_emoji::available() && !clusters(text).is_empty()
}

/// The byte ranges of the emoji clusters in `text`.
fn clusters(text: &str) -> Vec<Range<usize>> {
    // Every emoji, keycaps included, has a character from U+00A9 up; most
    // names have none, and skip the segmentation.
    if text.chars().all(|character| u32::from(character) < 0xA9) {
        return Vec::new();
    }
    let mut ranges = Vec::new();
    let mut at = 0;
    for piece in fastframe_emoji::pieces(text) {
        let (fastframe_emoji::Piece::Text(run) | fastframe_emoji::Piece::Emoji(run)) = piece;
        if matches!(piece, fastframe_emoji::Piece::Emoji(_)) {
            ranges.push(at..at + run.len());
        }
        at += run.len();
    }
    ranges
}

/// Makes the monochrome glyphs of each emoji in `galley` transparent, so a
/// colour picture can be painted in their place. Leaves the galley alone
/// when it holds no emoji or no colour font was found.
pub fn hide(galley: &mut Arc<Galley>) {
    if !fastframe_emoji::available() {
        return;
    }
    hide_clusters(galley);
}

fn hide_clusters(galley: &mut Arc<Galley>) {
    let ranges = clusters(&galley.job.text);
    if ranges.is_empty() {
        return;
    }
    let galley = Arc::make_mut(galley);
    for placed in &mut galley.rows {
        if !placed
            .row
            .glyphs
            .iter()
            .any(|glyph| in_cluster(&ranges, glyph.cluster))
        {
            continue;
        }
        let row = Arc::make_mut(&mut placed.row);
        let quads: Vec<u32> = row
            .glyphs
            .iter()
            .filter(|glyph| in_cluster(&ranges, glyph.cluster) && !glyph.uv_rect.is_nothing())
            .map(|glyph| glyph.first_vertex)
            .collect();
        let vertices = &mut row.visuals.mesh.vertices;
        for first in quads {
            let first = first as usize;
            for vertex in vertices.iter_mut().skip(first).take(4) {
                vertex.color = Color32::TRANSPARENT;
            }
        }
    }
}

fn in_cluster(ranges: &[Range<usize>], byte: u32) -> bool {
    ranges.iter().any(|range| range.contains(&(byte as usize)))
}

/// Each emoji's clusters and the rectangle its glyphs take, relative to the
/// galley, one per cluster and row.
fn placements(galley: &Galley) -> Vec<(String, Rect)> {
    let text = &galley.job.text;
    let ranges = clusters(text);
    let mut found = Vec::new();
    for range in &ranges {
        for placed in &galley.rows {
            let rect = placed
                .row
                .glyphs
                .iter()
                .filter(|glyph| range.contains(&(glyph.cluster as usize)))
                .map(|glyph| {
                    Rect::from_min_max(
                        Pos2::new(glyph.pos.x, 0.0),
                        Pos2::new(glyph.max_x(), placed.row.size.y),
                    )
                })
                .reduce(|a, b| a.union(b));
            if let Some(rect) = rect {
                found.push((
                    text[range.clone()].to_owned(),
                    rect.translate(placed.pos.to_vec2()),
                ));
            }
        }
    }
    found
}

/// Paints the colour pictures over a galley that [`hide`] prepared, drawn
/// at `origin`, clipped to `clip`.
pub fn paint(ui: &Ui, clip: Rect, galley: &Galley, origin: Pos2) {
    if !fastframe_emoji::available() {
        return;
    }
    let found = placements(galley);
    if found.is_empty() {
        return;
    }
    // fastframe paints with the ui's own painter, so a cell's narrower clip
    // is kept here: a picture the cell would cut is left out rather than
    // spilling into the next column.
    let clip = clip.intersect(ui.clip_rect()).expand(0.5);
    for (cluster, rect) in found {
        let rect = rect.translate(origin.to_vec2());
        if clip.contains_rect(rect) {
            fastframe_emoji::paint_cluster(ui, &cluster, rect);
        }
    }
}

/// Paints a galley with its emoji in colour: hidden, painted, then drawn
/// over. The same as `painter.galley` for a galley without emoji.
pub fn galley(ui: &Ui, painter: &Painter, origin: Pos2, mut galley: Arc<Galley>, color: Color32) {
    if !shows(&galley.job.text) {
        painter.galley(origin, galley, color);
        return;
    }
    hide(&mut galley);
    painter.galley(origin, galley.clone(), color);
    paint(ui, painter.clip_rect(), &galley, origin);
}

/// [`crate::bidi::paint_line`] with its emoji in colour.
#[allow(clippy::too_many_arguments)]
pub fn paint_line(
    ui: &Ui,
    painter: &Painter,
    left: f32,
    right: f32,
    y: f32,
    text: &str,
    font: egui::FontId,
    color: Color32,
) -> Rect {
    if !shows(text) {
        return crate::bidi::paint_line(painter, left, right, y, text, font, color);
    }
    let laid = crate::bidi::layout_line(painter, text, font, color);
    let rect = if crate::bidi::is_rtl(text) {
        egui::Align2::RIGHT_CENTER.anchor_size(egui::pos2(right, y), laid.size())
    } else {
        egui::Align2::LEFT_CENTER.anchor_size(egui::pos2(left, y), laid.size())
    };
    galley(ui, painter, rect.min, laid, color);
    rect
}

/// An egui label for `galley` with its emoji in colour. The label paints the
/// galley with the monochrome glyphs hidden; the pictures go on top.
pub fn label(ui: &mut Ui, mut galley: Arc<Galley>, sense: egui::Sense) -> egui::Response {
    let colour = shows(&galley.job.text);
    if colour {
        hide(&mut galley);
    }
    let response = ui.add(
        egui::Label::new(galley.clone())
            .selectable(false)
            .sense(sense),
    );
    if colour {
        let origin = crate::bidi::galley_pos(response.rect, &galley);
        paint(ui, response.rect.expand(2.0), &galley, origin);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    fn laid(ctx: &egui::Context, text: &str) -> Arc<Galley> {
        let mut galley = None;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            galley = Some(ui.painter().layout_no_wrap(
                text.to_owned(),
                crate::theme::regular(14.0),
                Color32::WHITE,
            ));
        });
        output.textures_delta.clear();
        galley.expect("laid out")
    }

    fn context() -> egui::Context {
        let ctx = egui::Context::default();
        crate::theme::install(&ctx);
        ctx
    }

    #[test]
    fn plain_names_are_never_segmented() {
        assert!(clusters("Daft Punk - One More Time").is_empty());
        assert!(clusters("Sigur Rós · Hoppípolla").is_empty());
        assert_eq!(clusters("Chill 🌊 vibes"), vec![6..10]);
    }

    #[test]
    fn a_joined_emoji_in_a_title_is_one_picture() {
        let ctx = context();
        let title = "Family 👨‍👩‍👧 road trip";
        let galley = laid(&ctx, title);
        let found = placements(&galley);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].0, "👨‍👩‍👧");
        assert!(found[0].1.width() > 0.0);
    }

    #[test]
    fn flags_keycaps_and_skin_tones_are_one_picture_each() {
        let ctx = context();
        let galley = laid(&ctx, "🇮🇹 Nove 9️⃣ Kasia 👋🏽");
        let found: Vec<String> = placements(&galley)
            .into_iter()
            .map(|(cluster, _)| cluster)
            .collect();
        assert_eq!(found, vec!["🇮🇹", "9️⃣", "👋🏽"]);
    }

    #[test]
    fn hiding_leaves_only_the_emoji_transparent() {
        let ctx = context();
        let mut galley = laid(&ctx, "A 🎧 B");
        let before = galley.rows[0].row.visuals.mesh.vertices.clone();
        hide_clusters(&mut galley);
        let row = &galley.rows[0].row;
        let hidden: Vec<char> = row
            .glyphs
            .iter()
            .filter(|glyph| !glyph.uv_rect.is_nothing())
            .filter(|glyph| {
                row.visuals.mesh.vertices[glyph.first_vertex as usize].color == Color32::TRANSPARENT
            })
            .map(|glyph| glyph.chr)
            .collect();
        assert_eq!(hidden, vec!['🎧']);
        assert_eq!(before.len(), row.visuals.mesh.vertices.len());
        // The layout itself is untouched: every glyph keeps its place.
        assert_eq!(laid(&ctx, "A 🎧 B").size(), galley.size());
    }

    #[test]
    fn without_a_colour_font_the_monochrome_face_draws_the_emoji() {
        // No colour emoji font is installed in this process (the default
        // setup on a test machine may or may not find one), so check the
        // rule itself: `hide` changes nothing unless a font is available,
        // and the bundled monochrome face draws a visible glyph.
        let ctx = context();
        let mut galley = laid(&ctx, "🎧");
        let glyph = galley.rows[0].row.glyphs[0];
        assert!(!glyph.uv_rect.is_nothing(), "the monochrome face draws it");
        if !fastframe_emoji::available() {
            let before = galley.rows[0].row.visuals.mesh.vertices.clone();
            hide(&mut galley);
            assert_eq!(before, galley.rows[0].row.visuals.mesh.vertices);
        }
    }
}
