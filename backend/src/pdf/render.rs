#![allow(non_snake_case)]

use euclid::Size2D;

use crate::backend::Backend;

use crate::parameters::UserStepsOptions;
use crate::point_collection::Kinds;
use crate::svgtable::waypoints_to_svg;
use crate::waypoint::decimate;
use crate::{track, waypoint};

use pdf_writer::{Chunk, Content, Finish, Name, Pdf, Rect, Ref};
use svg2pdf::usvg::{self, Tree};
use svg2pdf::ConversionOptions;

use std::collections::HashMap;
use std::fs;

pub struct TableInfo<'a> {
    pub track: &'a track::Track,
    pub waypoints: Vec<waypoint::Waypoint>,
    pub user_steps_options: UserStepsOptions,
    pub elevation_gain: f64,
}

fn points_table_svg(table_info: TableInfo) -> String {
    waypoints_to_svg(table_info, 3.25f64)
}

const PAGE_WIDTH: f32 = 595.2756;
const PAGE_HEIGHT: f32 = 841.8898;
const PAGE_MARGIN: f32 = 8.503937;
const CONTENT_WIDTH: f32 = PAGE_WIDTH - 2.0 * PAGE_MARGIN;
const CELL_GAP: f32 = 12.0;
const PROFILE_GAP: f32 = 8.0;
const ROW_GAP: f32 = 13.0;
const TABLE_SEPARATOR_WIDTH: f32 = 1.0;
const TABLES_PER_PAGE: usize = 2;

#[derive(Clone)]
struct XObjectEntry {
    name: Vec<u8>,
    reference: Ref,
}

struct SvgGraphic {
    entry: XObjectEntry,
    width: f32,
    height: f32,
}

struct PageState {
    page_ref: Ref,
    content_ref: Ref,
    content: Content,
    xobjects: Vec<XObjectEntry>,
    y_cursor: f32,
    tables_on_page: usize,
}

impl PageState {
    fn new(page_ref: Ref, content_ref: Ref) -> Self {
        let mut content = Content::new();
        content.set_line_width(TABLE_SEPARATOR_WIDTH);
        Self {
            page_ref,
            content_ref,
            content,
            xobjects: Vec::new(),
            y_cursor: PAGE_HEIGHT - PAGE_MARGIN,
            tables_on_page: 0,
        }
    }

    fn add_xobject(&mut self, entry: XObjectEntry) -> usize {
        self.xobjects.push(entry);
        self.xobjects.len() - 1
    }

    fn draw_separator(&mut self, y: f32) {
        self.content.move_to(PAGE_MARGIN, y);
        self.content
            .line_to(PAGE_MARGIN + CONTENT_WIDTH, y)
            .stroke();
    }

    fn place_xobject(&mut self, idx: usize, x: f32, y: f32, width: f32, height: f32) {
        let name = Name(&self.xobjects[idx].name);
        PdfComposer::draw_xobject(&mut self.content, name, x, y, width, height);
    }
}

struct PdfComposer {
    pdf: Pdf,
    next_id: i32,
    catalog_ref: Ref,
    page_tree_ref: Ref,
    page_refs: Vec<Ref>,
    current_page: Option<PageState>,
    options: usvg::Options<'static>,
    conversion: ConversionOptions,
    xobject_serial: usize,
}

impl PdfComposer {
    fn new(options: usvg::Options<'static>) -> Self {
        let mut composer = PdfComposer {
            pdf: Pdf::new(),
            next_id: 1,
            catalog_ref: Ref::new(1),
            page_tree_ref: Ref::new(2),
            page_refs: Vec::new(),
            current_page: None,
            options,
            conversion: ConversionOptions::default(),
            xobject_serial: 1,
        };
        composer.catalog_ref = composer.alloc_ref();
        composer.page_tree_ref = composer.alloc_ref();
        composer
    }

    fn alloc_ref(&mut self) -> Ref {
        let id = self.next_id;
        self.next_id += 1;
        Ref::new(id)
    }

    fn ensure_page(&mut self) {
        if self.current_page.is_none() {
            self.start_new_page();
        }
    }

    fn start_new_page(&mut self) {
        let page_ref = self.alloc_ref();
        let content_ref = self.alloc_ref();
        self.current_page = Some(PageState::new(page_ref, content_ref));
    }

    fn finish_current_page(&mut self) {
        if let Some(page) = self.current_page.take() {
            let content_stream = page.content.finish();
            self.pdf.stream(page.content_ref, &content_stream);
            let mut page_obj = self.pdf.page(page.page_ref);
            page_obj.media_box(Rect::new(0.0, 0.0, PAGE_WIDTH, PAGE_HEIGHT));
            page_obj.parent(self.page_tree_ref);
            page_obj.contents(page.content_ref);
            {
                let mut resources = page_obj.resources();
                if !page.xobjects.is_empty() {
                    let mut xobjects = resources.x_objects();
                    for entry in &page.xobjects {
                        xobjects.pair(Name(&entry.name), entry.reference);
                    }
                    xobjects.finish();
                }
                resources.finish();
            }
            page_obj.finish();
            self.page_refs.push(page.page_ref);
        }
    }

    fn prepare_page(&mut self, required_height: f32) {
        self.ensure_page();
        let needs_new_page = {
            let page = self.current_page.as_ref().unwrap();
            page.tables_on_page >= TABLES_PER_PAGE
                || (page.y_cursor - required_height) < PAGE_MARGIN
        };
        if needs_new_page {
            self.finish_current_page();
            self.start_new_page();
        }
    }

    fn fit_to_width(graphic: &SvgGraphic, width: f32) -> (f32, f32) {
        if graphic.width == 0.0 {
            return (width, width);
        }
        let scale = width / graphic.width;
        (width, graphic.height * scale)
    }

    fn fit_table(graphic: &SvgGraphic, max_width: f32) -> (f32, f32) {
        if graphic.width > 0.0 && graphic.width <= max_width {
            return (graphic.width, graphic.height);
        }
        Self::fit_to_width(graphic, max_width)
    }

    fn scale_map(graphic: &SvgGraphic, max_width: f32, target_height: f32) -> (f32, f32) {
        if graphic.width == 0.0 || graphic.height == 0.0 {
            return (max_width, target_height);
        }
        if target_height > 0.0 {
            let height_scale = target_height / graphic.height;
            let width = graphic.width * height_scale;
            if width <= max_width {
                return (width, target_height);
            }
        }
        let scale = max_width / graphic.width;
        (max_width, graphic.height * scale)
    }

    fn add_table(&mut self, profile_svg: &str, map_svg: &str, table_svg: &str) {
        let profile = self.load_svg(profile_svg);
        let map = self.load_svg(map_svg);
        let points = self.load_svg(table_svg);

        let (profile_width, profile_height) = Self::fit_to_width(&profile, CONTENT_WIDTH);
        //let column_width = (CONTENT_WIDTH - CELL_GAP) / 2.0;
        let (map_width, map_height) =
            Self::scale_map(&map, profile_height * 1.25, profile_height * 1.25);
        let (table_width, table_height) =
            Self::fit_table(&points, CONTENT_WIDTH - CELL_GAP - map_width);
        let separator_height = map_height.max(table_height);

        let SvgGraphic {
            entry: profile_entry,
            ..
        } = profile;
        let SvgGraphic {
            entry: map_entry, ..
        } = map;
        let SvgGraphic {
            entry: points_entry,
            ..
        } = points;

        let row_height = map_height.max(table_height);
        let required_height = profile_height + PROFILE_GAP + row_height + ROW_GAP;
        self.prepare_page(required_height);

        let page = self.current_page.as_mut().unwrap();
        page.draw_separator(page.y_cursor);
        page.y_cursor -= 2.0;

        let profile_bottom = page.y_cursor - profile_height;
        let profile_idx = page.add_xobject(profile_entry);
        page.place_xobject(
            profile_idx,
            PAGE_MARGIN - 4.0,
            profile_bottom,
            profile_width,
            profile_height,
        );
        page.y_cursor = profile_bottom - PROFILE_GAP;
        let row_separator_y = profile_bottom - (PROFILE_GAP / 2.0);
        page.draw_separator(row_separator_y);

        let row_bottom = page.y_cursor - row_height;
        let map_idx = page.add_xobject(map_entry);
        let table_idx = page.add_xobject(points_entry);

        let map_left = PAGE_MARGIN; // + ((column_width - map_width) / 2.0).max(0.0);
        let map_bottom = row_bottom + (row_height - map_height) / 2.0;
        let table_left =
            PAGE_MARGIN + map_width + (((CONTENT_WIDTH - map_width) - table_width) / 2.0).max(0.0);
        let table_bottom = row_bottom + (row_height - table_height) / 2.0;

        page.place_xobject(map_idx, map_left, map_bottom, map_width, map_height);
        page.place_xobject(
            table_idx,
            table_left,
            table_bottom,
            table_width,
            table_height,
        );

        page.y_cursor = row_bottom;
        let separator_left = PAGE_MARGIN + map_width + (CELL_GAP / 2.0);
        page.content.move_to(separator_left, row_separator_y);
        page.content
            .line_to(separator_left, row_bottom - 3.0)
            .stroke();
        page.draw_separator(row_bottom - 3.0);

        page.content
            .move_to(PAGE_MARGIN, profile_bottom + profile_height + 2.0);
        page.content.line_to(PAGE_MARGIN, row_bottom - 3.0).stroke();

        page.content.move_to(
            PAGE_MARGIN + CONTENT_WIDTH,
            profile_bottom + profile_height + 2.0,
        );
        page.content
            .line_to(PAGE_MARGIN + CONTENT_WIDTH, row_bottom - 3.0)
            .stroke();

        page.y_cursor -= ROW_GAP;
        page.tables_on_page += 1;
    }

    fn draw_xobject(
        content: &mut Content,
        name: Name<'_>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        content.save_state();
        content.transform([width, 0.0, 0.0, height, x, y]);
        content.x_object(name);
        content.restore_state();
    }

    fn load_svg(&mut self, svg: &str) -> SvgGraphic {
        let tree = Tree::from_str(svg, &self.options).expect("invalid svg data");
        let size = tree.size();
        let (chunk, root_ref) =
            svg2pdf::to_chunk(&tree, self.conversion).expect("failed to convert svg");
        let entry = self.embed_chunk(chunk, root_ref);
        SvgGraphic {
            entry,
            width: size.width() as f32,
            height: size.height() as f32,
        }
    }

    fn embed_chunk(&mut self, chunk: Chunk, root: Ref) -> XObjectEntry {
        let mut map = HashMap::new();
        let mut next_id = self.next_id;
        let remapped = chunk.renumber(|old| {
            *map.entry(old).or_insert_with(|| {
                let r = Ref::new(next_id);
                next_id += 1;
                r
            })
        });
        self.next_id = next_id;
        let reference = *map.get(&root).expect("missing root reference");
        self.pdf.extend(&remapped);
        let name = format!("X{}", self.xobject_serial).into_bytes();
        self.xobject_serial += 1;
        XObjectEntry { name, reference }
    }

    fn finish(mut self) -> Vec<u8> {
        if self.current_page.is_some() || self.page_refs.is_empty() {
            self.finish_current_page();
        }
        if self.page_refs.is_empty() {
            self.start_new_page();
            self.finish_current_page();
        }
        self.pdf.catalog(self.catalog_ref).pages(self.page_tree_ref);
        let count = self.page_refs.len() as i32;
        self.pdf
            .pages(self.page_tree_ref)
            .kids(self.page_refs.iter().copied())
            .count(count);
        self.pdf.finish()
    }
}

fn link(profilesvg: &str, mapsvg: &str, points_table_svg: &String, document: &mut PdfComposer) {
    document.add_table(profilesvg, mapsvg, points_table_svg);
}

pub async fn make_pdf_document(backend: &Backend, kinds: &Kinds) -> Vec<u8> {
    let mut options: usvg::Options<'static> = usvg::Options::default();
    options.fontdb_mut().load_system_fonts();
    super::fonts::register_libertinus_fonts(options.fontdb_mut()).await;
    let mut document = PdfComposer::new(options);
    let debug = backend.get_parameters().debug;
    let fsegments = backend.segments();
    let segments: Vec<_> = fsegments
        .iter()
        .map(|f| backend.make_segment_data(&f))
        .collect();

    for segment in &segments {
        let range = segment.range();
        if range.is_empty() {
            continue;
        }
        let profile_size = Size2D::new(1000, 300);
        let map_size = Size2D::new(400, 400);
        let both = backend.render_segment_map_profile(
            &segment.segment,
            &map_size,
            &profile_size,
            kinds.clone(),
        );
        let [rendered_map, rendered_profile]: [_; 2] = both.try_into().unwrap();
        let waypoints = decimate(&segment.segment, &rendered_profile.waypoints, 15);
        let user_steps_options = backend.get_parameters().user_steps_options.clone();
        let elevation_gain = backend.d().track.elevation_gain_on_range(&range);
        let table_info = TableInfo {
            track: &backend.d().track,
            waypoints: waypoints.clone(),
            user_steps_options,
            elevation_gain,
        };
        let table_svg = points_table_svg(table_info);
        if debug {
            let f = format!("/tmp/segment-{}.svg", segment.id());
            fs::write(&f, &rendered_profile.svg).unwrap();
            let f = format!("/tmp/map-{}.svg", segment.id());
            fs::write(&f, &rendered_map.svg).unwrap();
        }
        link(
            &rendered_profile.svg,
            &rendered_map.svg,
            &table_svg,
            &mut document,
        );
        if range.end == backend.d().track.len() {
            break;
        }
    }
    document.finish()
}
