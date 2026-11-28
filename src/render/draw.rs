use parley::{Layout, Alignment, AlignmentOptions, FontWeight, GenericFamily, PositionedLayoutItem, StyleProperty};
use vello::{kurbo::{Affine, Line as KurboLine, Point, Rect, Stroke as KurboStroke, Vec2}, peniko::{Color, Fill as KurboFill, ImageBrush, ImageData, ImageSampler, color::{AlphaColor, Srgb}}};

use crate::{render::{ColorBrush, Renderer}, space::{Context, Contexted}};

pub type Stroke = (f64, Color);
pub type Fill = Color;

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
}

pub struct DrawRectArgs<T:Context> {
    pub rect: Contexted<Rect, T>,
    pub style: Style,
}

pub struct DrawLineArgs<T:Context> {
    pub start: Contexted<Point, T>,
    pub end: Contexted<Point, T>,
    pub stroke: Stroke,
}

pub struct BuildTextArgs<'a> {
    pub text: &'a str,
    pub font_size: f32,
    pub color: Color,
    pub center: bool,
}

pub struct TextLayout {
    layout: Layout<ColorBrush>, center: bool,
}

impl TextLayout {
    #[allow(unused)]
    pub fn height(&self) -> f32 {
        self.layout.height()
    }
}

pub struct DrawImageArgs<T:Context> {
    pub rect: Contexted<Rect,T>,
    pub image: ImageData,
    pub alpha: f32,
}

impl Renderer {
    pub fn draw_rect<T:Context>(&mut self, mut args:DrawRectArgs<T>, state:T::State<'_>) {
        if let Some(color) = args.style.fill {
            self.scene.fill(KurboFill::NonZero, T::transform_from(state), color, None, args.rect.as_inner());
        }
        if let Some((width, color)) = args.style.stroke {
            self.scene.stroke(&KurboStroke::new(width), T::transform_from(state), color, None, args.rect.as_inner());
        }
    }

    #[allow(unused)]
    pub fn draw_line<T:Context>(&mut self, args:DrawLineArgs<T>, state:T::State<'_>) {
        let (width, color) = args.stroke;
        self.scene.stroke(
            &KurboStroke::new(width), T::transform_from(state), color, None,
            &KurboLine { p0: args.start.to_inner(), p1: args.end.to_inner() }
        );
    }

    #[must_use]
    pub fn build_text(&mut self, args:BuildTextArgs) -> TextLayout {
        let mut builder = self.layout_cx.ranged_builder(&mut self.font_cx, args.text, 1., true);
        builder.push_default(GenericFamily::SystemUi);
        builder.push_default(StyleProperty::FontSize(args.font_size));
        builder.push_default(FontWeight::new(400.));
        builder.push_default(StyleProperty::Brush(ColorBrush(args.color)));
        let mut layout = builder.build(args.text);
        layout.break_all_lines(None);
        layout.align(if args.center { Alignment::Center } else { Alignment::Start }, AlignmentOptions::default());
        TextLayout { layout, center: args.center }
    }

    pub fn draw_text<T:Context>(&mut self, position:Contexted<Point, T>, layout:&TextLayout, state:T::State<'_>) {
        let mut translate = position.to_inner().to_vec2();
        if layout.center {
            translate += Vec2::new(-0.5*layout.layout.full_width() as f64, -0.5*layout.layout.height() as f64);
        }

        for line in layout.layout.lines() {
            for item in line.items() {
                match item {
                    PositionedLayoutItem::GlyphRun(glyph_run) => {
                        let run = glyph_run.run();
                        let coords = bytemuck::cast_slice::<_, i16>(run.normalized_coords());
                        self.scene.draw_glyphs(run.font())
                            .font_size(run.font_size())
                            .normalized_coords(coords)
                            .transform(T::transform_from(state).pre_translate(translate))
                            .brush(glyph_run.style().brush)
                            .draw(KurboFill::NonZero, glyph_run.positioned_glyphs().map(|g| {
                                vello::Glyph { id: g.id, x: g.x, y: g.y }
                            }));
                    },
                    PositionedLayoutItem::InlineBox(inline_box) => {
                        let (x0, y0) = (inline_box.x as f64, inline_box.y as f64);
                        let (x1, y1) = (x0 + inline_box.width as f64, y0 + inline_box.height as f64);
                        self.scene.fill(KurboFill::EvenOdd, Affine::default(), AlphaColor::<Srgb>::BLACK, None, &Rect {
                            x0, y0, x1, y1
                        });
                    },
                }
            }
        }
    }

    pub fn draw_text_immediate<T:Context>(&mut self, position:Contexted<Point, T>, args:BuildTextArgs, state:T::State<'_>) {
        let layout = self.build_text(args);
        self.draw_text::<T>(position, &layout, state);
    }

    pub fn draw_image<T:Context>(&mut self, args:DrawImageArgs<T>, state:T::State<'_>) {
        let rect = args.rect.to_screen(state).to_inner();
        self.scene.draw_image(ImageBrush {
            image: &args.image, sampler: ImageSampler { alpha: args.alpha, quality: vello::peniko::ImageQuality::Low, ..Default::default() }
        }, Affine::translate(rect.origin().to_vec2()).pre_scale_non_uniform(rect.width() / args.image.width as f64, rect.height() / args.image.height as f64));
    }

}

