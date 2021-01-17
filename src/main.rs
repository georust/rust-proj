#[macro_use]
extern crate approx;

use proj_sys::{
    proj_area_destroy, proj_cleanup, proj_context_create, proj_context_destroy, proj_create,
    proj_destroy, proj_errno, proj_errno_reset, proj_trans, PJconsts, PJ_AREA,
    PJ_CONTEXT, PJ_COORD, PJ_DIRECTION_PJ_FWD, PJ_XY,
};
use std::ffi::CString;
use std::str;

pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub struct Proj {
    c_proj: *mut PJconsts,
    ctx: *mut PJ_CONTEXT,
    area: Option<*mut PJ_AREA>,
}

impl Drop for Proj {
    fn drop(&mut self) {
        unsafe {
            if let Some(area) = self.area {
                proj_area_destroy(area)
            }
            proj_destroy(self.c_proj);
            proj_context_destroy(self.ctx);
            // NB do NOT call until proj_destroy and proj_context_destroy have both returned:
            // https://proj.org/development/reference/functions.html#c.proj_cleanup
            proj_cleanup()
        }
    }
}

fn project(definition: &str, point: Point) -> Point {
    let ctx = unsafe { proj_context_create() };
    let c_definition = CString::new(definition).unwrap();
    let new_c_proj = unsafe { proj_create(ctx, c_definition.as_ptr()) };
    let proj = Proj {
        c_proj: new_c_proj,
        ctx,
        area: None,
    };
    let coords = PJ_XY { x: point.x, y: point.y };
    let (new_x, new_y, err) = unsafe {
        proj_errno_reset(proj.c_proj);
        let trans = proj_trans(proj.c_proj, PJ_DIRECTION_PJ_FWD, PJ_COORD { xy: coords });
        (trans.xy.x, trans.xy.y, proj_errno(proj.c_proj))
    };
    assert_eq!(err, 0, "error: {}", err);
    Point { x: new_x, y: new_y }
}

fn main() {
    let t = project(
        "+proj=sterea +lat_0=46 +lon_0=25 +k=0.99975 +x_0=500000 +y_0=500000
        +ellps=krass +towgs84=33.4,-146.6,-76.3,-0.359,-0.053,0.844,-0.84 +units=m +no_defs",
        Point {
            x: 0.436332,
            y: 0.802851,
        },
    );
    assert_relative_eq!(t.x, 500119.7035366755, epsilon = 1e-5);
    assert_relative_eq!(t.y, 500027.77901023754, epsilon = 1e-5);
}
