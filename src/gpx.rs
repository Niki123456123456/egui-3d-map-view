pub fn open() -> poll_promise::Promise<GpxRoute> {
    let (sender, promise) = poll_promise::Promise::new();
    let task = rfd::AsyncFileDialog::new().pick_file();
    crate::http::execute(async move {
        let file = task.await;
        if let Some(file) = file {
            let bytes = file.read().await;
            let cursor = std::io::Cursor::new(bytes);
            if let Ok(gpx) = gpx::read(cursor) {
                let mut points = vec![];
                for t in gpx.tracks.iter() {
                    for s in t.segments.iter() {
                        for p in s.points.iter() {
                            let lat = p.point().lat();
                            let lon = p.point().lng();
                            if let Some(ele) = p.elevation {
                                let xyz = crate::maps::latlon_to_xyz(lat, lon, ele);
                                points.push(Point { lat, lon, ele: ele + 12620., xyz });
                            }
                        }
                    }
                }
                println!("got {}", points.len());

                sender.send(GpxRoute { gpx, points });
            }
        }
    });
    return promise;
}

#[derive(Clone)]
pub struct GpxRoute {
    pub gpx: gpx::Gpx,
    pub points: Vec<Point>,
}

pub struct GpxRouteGPU {
    pub route: GpxRoute,
    pub mesh: crate::lines::LineMesh,
}

impl GpxRouteGPU {
    pub fn new(route: GpxRoute, context: &three_d::Context) -> Self {
        let mut lines = vec![];
        for (i,p) in route.points.iter().enumerate() {
            if i > 0 {
                let previous = &route.points[i-1]; lines.push(previous);
                 lines.push(p);
            }
        }
        let mesh = crate::lines::LineMesh::from_vector(
            context,
            lines
                .iter()
                .map(|x| three_d::vec3(x.xyz.x as f32, x.xyz.y as f32, x.xyz.z as f32))
                .collect(),
        );
        Self { route, mesh }
    }
}

#[derive(Clone)]
pub struct Point {
    pub lat: f64,
    pub lon: f64,
    pub ele: f64,
    pub xyz: glam::DVec3,
}
