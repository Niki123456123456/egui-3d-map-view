use super::*;

pub struct OrientedBoundingBox {
    pub center: glam::DVec3,
    pub half_axes: glam::DMat3,
    pub inverse_half_axes: glam::DMat3,
    pub lengths: glam::DVec3,
}

pub fn equals_epsilon(
    left: glam::DVec3,
    right: glam::DVec3,
    relative_epsilon: f64,
    absolute_epsilon: f64,
) -> bool {
    let diff = (left - right).abs();

    if diff.x <= absolute_epsilon || diff.y <= absolute_epsilon || diff.z <= absolute_epsilon {
        return true;
    }
    if diff.x <= relative_epsilon_to_absolute(left.x, left.x, relative_epsilon)
        || diff.y <= relative_epsilon_to_absolute(left.y, left.y, relative_epsilon)
        || diff.z <= relative_epsilon_to_absolute(left.z, left.z, relative_epsilon)
    {
        return true;
    }
    return false;
}

pub fn relative_epsilon_to_absolute(a: f64, b: f64, relative_epsilon: f64) -> f64 {
    return relative_epsilon * a.abs().max(b.abs());
}

impl OrientedBoundingBox {
    pub fn new(center: glam::DVec3, half_axes: glam::DMat3) -> Self {
        Self {
            center,
            half_axes,
            inverse_half_axes: half_axes.inverse(),
            lengths: glam::dvec3(
                half_axes.col(0).length(),
                half_axes.col(1).length(),
                half_axes.col(2).length(),
            ) * 2.,
        }
    }

    pub fn transform(&self, transformation: glam::DMat4) -> Self {
        Self::new(
            (transformation * glam::dvec4(self.center.x, self.center.y, self.center.z, 1.)).xyz(),
            glam::dmat3(
                transformation.col(0).xyz(),
                transformation.col(1).xyz(),
                transformation.col(1).xyz(),
            ) * self.half_axes,
        )
    }

    pub fn intersect_plane(&self, plane: &Plane) -> CullingResult {
        let rad_effective = self.half_axes.col(0).dot(plane.normal).abs()
            + self.half_axes.col(1).dot(plane.normal).abs()
            + self.half_axes.row(2).dot(plane.normal).abs();
        let distance_to_plane = plane.normal.dot(self.center + plane.d);
        if distance_to_plane <= -rad_effective {
            return CullingResult::Outside;
        }
        if distance_to_plane >= rad_effective {
            return CullingResult::Inside;
        }
        return CullingResult::Intersecting;
    }

    pub fn compute_distance_squared_to_position(&self, position: glam::DVec3) -> f64 {
        let offset = position - self.center;

        let mut u = self.half_axes.col(0);
        let mut v = self.half_axes.col(1);
        let mut w = self.half_axes.col(2);

        let uHalf = u.length();
        let vHalf = v.length();
        let wHalf = w.length();

        let uValid = uHalf > 0.;
        let vValid = vHalf > 0.;
        let wValid = wHalf > 0.;

        let mut numberOfDegenerateAxes = 0;
        if (uValid) {
            u /= uHalf;
        } else {
            numberOfDegenerateAxes += 1;
        }

        if (vValid) {
            v /= vHalf;
        } else {
            numberOfDegenerateAxes += 1;
        }

        if (wValid) {
            w /= wHalf;
        } else {
            numberOfDegenerateAxes += 1;
        }
        let mut validAxis1 = glam::DVec3::ZERO;
        let mut validAxis2 = glam::DVec3::ZERO;
        let mut validAxis3 = glam::DVec3::ZERO;

        if (numberOfDegenerateAxes == 1) {
            let mut degenerateAxis = u;
            validAxis1 = v;
            validAxis2 = w;

            if (!vValid) {
                degenerateAxis = v;
                validAxis1 = u;
            } else if (!wValid) {
                degenerateAxis = w;
                validAxis2 = u;
            }

            validAxis3 = validAxis1.cross(validAxis2);

            if (!uValid) {
                u = validAxis3;
            } else if (!vValid) {
                v = validAxis3;
            } else {
                w = validAxis3;
            }
        } else if (numberOfDegenerateAxes == 2) {
            if (uValid) {
                validAxis1 = u;
            } else if (vValid) {
                validAxis1 = v;
            } else {
                validAxis1 = w;
            }

            let mut crossVector = glam::dvec3(0., 1., 0.);
            if (equals_epsilon(validAxis1, crossVector, 1e-3, 1e-3)) {
                crossVector = glam::dvec3(1., 0., 0.);
            }

            validAxis2 = validAxis1.cross(crossVector).normalize();
            validAxis3 = validAxis1.cross(validAxis2).normalize();

            if (uValid) {
                v = validAxis2;
                w = validAxis3;
            } else if (vValid) {
                w = validAxis2;
                u = validAxis3;
            } else if (wValid) {
                u = validAxis2;
                v = validAxis3;
            }
        } else if (numberOfDegenerateAxes == 3) {
            u = glam::dvec3(1., 0., 0.);
            v = glam::dvec3(0., 1., 0.);
            w = glam::dvec3(0., 0., 1.);
        }

        let pPrime = glam::dvec3(offset.dot(u), offset.dot(v), offset.dot(w));

        let mut distanceSquared = 0.0;
        let mut d = 0.;

        if (pPrime.x < -uHalf) {
            d = pPrime.x + uHalf;
            distanceSquared += d * d;
        } else if (pPrime.x > uHalf) {
            d = pPrime.x - uHalf;
            distanceSquared += d * d;
        }

        if (pPrime.y < -vHalf) {
            d = pPrime.y + vHalf;
            distanceSquared += d * d;
        } else if (pPrime.y > vHalf) {
            d = pPrime.y - vHalf;
            distanceSquared += d * d;
        }

        if (pPrime.z < -wHalf) {
            d = pPrime.z + wHalf;
            distanceSquared += d * d;
        } else if (pPrime.z > wHalf) {
            d = pPrime.z - wHalf;
            distanceSquared += d * d;
        }

        return distanceSquared;
    }
}