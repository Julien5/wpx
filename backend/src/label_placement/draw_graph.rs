use crate::bbox::BoundingBox;
use crate::math::Point2D;

pub struct Graphic {
    elements: Vec<String>,
    dir: String,
}

#[allow(dead_code)]
pub fn newdir() -> String {
    use std::path::Path;
    let dir = "/tmp/graphs/";
    // Find the next available filename
    let mut n = 0;
    let dir = loop {
        let path = format!("{}/graph-{:02}", dir, n);
        if !Path::new(&path).exists() {
            break path;
        }
        n += 1;
    };
    dir
}

impl Graphic {
    pub fn new(dir: String) -> Self {
        Self {
            elements: Vec::new(),
            dir,
        }
    }

    /*
    <text font-size="16.0" id="30/wp/text" text-anchor="start" x="4.000" y="16.000">
    </text>
         */
    pub fn add_text(&mut self, p: &Point2D, text: &str) {
        // Draw rectangle
        let rect = format!(
            r#"<text font-size="14.0" text-anchor="start" x="{}" y="{}">{}</text>"#,
            p.x, p.y, text
        );
        self.elements.push(rect);
    }

    pub fn add_boundingbox(&mut self, bbox: &BoundingBox, color: &str, swidth: i32) {
        // Draw rectangle
        let min = bbox.get_min();
        let max = bbox.get_max();
        let width = max.x - min.x;
        let height = max.y - min.y;

        let rect = format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="{}" stroke-width="{}"/>"#,
            min.x, min.y, width, height, color, swidth
        );
        self.elements.push(rect);
    }

    pub fn add_dot(&mut self, point: &Point2D) {
        let circle = format!(
            r#"<circle cx="{}" cy="{}" r="3" fill="red"/>"#,
            point.x, point.y
        );
        self.elements.push(circle);
    }

    pub fn add_stroke(&mut self, p1: &Point2D, p2: &Point2D) {
        let line = format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="blue" stroke-width="2"/>"#,
            p1.x, p1.y, p2.x, p2.y
        );
        self.elements.push(line);
    }

    pub fn render(&self) -> String {
        let width = 1000;
        let height = 800;
        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}" width="{}" height="{}">"#,
            0, 0, width, height, width, height
        );
        svg.push('\n');

        for element in &self.elements {
            svg.push_str("  ");
            svg.push_str(element);
            svg.push('\n');
        }

        svg.push_str("</svg>");
        svg
    }

    pub fn save(&self) {
        use std::fs;
        use std::io::Write;
        use std::path::Path;
        let content = self.render();

        // Ensure the directory exists
        fs::create_dir_all(&self.dir).expect("could not make dirs");

        // Find the next available filename
        let mut n = 0;
        let filepath = loop {
            let path = format!("{}/graph-{:04}.svg", self.dir, n);
            if !Path::new(&path).exists() {
                break path;
            }
            n += 1;
        };

        log::trace!("create {}", filepath);
        // Write the content to the file
        let mut file = fs::File::create(&filepath).expect("could not create file");
        file.write_all(content.as_bytes())
            .expect("could not write content to file");
    }
}
