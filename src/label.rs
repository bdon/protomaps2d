// a very primitive label collider,
// that does a linear search over drawn labels,
// rejecting any that intersect.
pub struct Collider {
    pub bboxes:Vec<((f64, f64),(f64, f64))>
}

impl Collider {
    pub fn add(&mut self, topleft: (f64, f64), bottomright: (f64, f64)) -> bool {
        for bbox in &self.bboxes {
            // x axis
            if bottomright.0 < (bbox.0).0 {
                continue
            }
            if topleft.0 > (bbox.1).0 {
                continue
            }

            // y axis
            if bottomright.1 < (bbox.0).1 {
                continue
            }
            if topleft.1 > (bbox.1).1 {
                continue
            }

            return false;
        } 
        self.bboxes.push((topleft,bottomright));
        return true;
    }
}
