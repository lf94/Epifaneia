const workplane = () => ({
  pos: [0,0],     // Current position of the "pen"
  pts: [[0,0]],   // List of points to draw the polygon / straight lines
  stk: [nothing], // Stack of additional shapes to union such as arc and spline
  nrm: [0,0]      // The normal of the previous edge
});

const line = (point_relative, plane) => {
  const p = plane.pos + point_relative;
  return { ...plane, pos: p, pts: concat [plane.pts, [p]] };
};

const line_to = (point_absolute, plane) => {
  const p = point_absolute,
  return { ...plane, pos: p, pts: concat [plane.pts, [p]] };
};

const v_line = (y_relative, plane) => {
  const p = [plane.pos.[X], plane.pos.[Y] + y_relative];
  return { ...plane, pos: p, pts: concat [plane.pts, [p]], nrm: [0, sign y_relative] };
};

const v_line_to = (y_absolute, plane) => {
  const p = [plane.pos.[X], y_absolute];
  return { ...plane, pos: p, pts: concat [plane.pts, [p]], nrm: [0, sign y_absolute] };
};

const h_line = (x_relative, plane) => {
  const p = [plane.pos.[X] + x_relative, plane.pos.[Y]];
  return { ...plane, pos: p, pts: concat [plane.pts, [p]], nrm: [sign x_relative, 0] };
};

const h_line_to = (x_absolute, plane) => {
  const p = [x_absolute, plane.pos.[Y]];
  return { ...plane, pos: p, pts: concat [plane.pts, [p]], nrm: [sign x_absolute, 0]};
};

const polar_line = ({ distance, angle }, plane) => {
  const nrm = cis angle;
  const p = plane.pos + (nrm * distance);
  return { ...plane, pos: p, pts: concat [plane.pts, [p]], nrm: nrm };
};

const polar_line_to = ({ distance, angle }, plane) => {
  const nrm = cis angle;
  const p = nrm * distance;
  return { ...plane, pos: p, pts: concat [plane.pts, [p]], nrm: nrm };
};

const move_rel = (point_relative, plane) => {
  const p = plane.pos + point_relative;
  return { ...plane, pos: p, pts: plane.pts };
};

const move_abs = (point_absolute, plane) => {
  const p = plane.pos + point_absolute;
  return { ...plane, pos: p, pts: plane.pts };
};

//
// The math is too computationally heavy for a single threePointArc circle.
// threePointArc [point_middle, point_end_relative] plane
//

const sagitta_arc = ([sag, point_end_relative], plane) =>
  sagitta_arc_to([sag, point_end_relative + plane.pos], plane);

const sagitta_arc_to = ([sag, point_end_absolute], plane) => {
  let p = point_end_absolute;
  let pn = (plane.pos - p);
  let chord = mag(pn);
  let nrm = pn / chord;
  let pm = (plane.pos + p) / 2;
  let l = chord / 2;
  let r = (sag^2 + l^2) / (2*sag);
  let v = [-pn.[Y], pn.[X]];
  let vn = v / (mag v);
  let o = pm + ((r - sag)*-vn);

  let shape = circle(r*2)
    >> move [...o, 0]
    >> into difference [
      half_plane { d: (r - sag), normal: vn }
      >> move [...o, 0]
  ];
  let item;

  return {
    ...plane,
    pos: p,
    pts: plane.pts.concat([p]),
    stk: plane.stk.concat([item]),
    nrm: nrm
  };
};

const radius_arc = ([r, point_end_relative], plane) =>
  radius_arc_to([r, point_end_relative + plane.pos], plane);

const radius_arc_to = ([r, point_end_absolute], plane) => {
  const chord = mag(plane.pos - point_end_absolute);
  const l = chord / 2;
  const sag = r - sqrt(r^2 - l^2);
  return sagitta_arc([sag, point_end_absolute], plane);
};

const tangent_arc_point = (point_end_relative, plane) =>
  tangent_arc_point_to((point_end_relative + plane.pos), plane);

const tangent_arc_point_to = (point_end_absolute, plane) => {
  let p = point_end_absolute;
  let Pa = plane.pos;
  let Pb = point_end_absolute;
  let PaPb = Pb - Pa;
  let nrm = PaPb / (mag PaPb);
  let T = plane.nrm;
  let s = dot[T, nrm];
  let d = mag (PaPb / cos(asin(s)));
  let r = d/2;
  let n = sign ((mag T) * (mag nrm) * s);
  let cn = if (n >= 0) [-T.[Y], T.[X]] else T;
  let c = Pa + (cn * n *r);
  let vn = -[-nrm.[Y], nrm.[X]];
  let l = (mag PaPb) / 2;
  let nrm_next = (Pb - c) / (mag (Pb - c));
  let sag = r - sqrt(r^2 - l^2);

  // TODO: how to pass this along
  // let shape = circle d
  //    >> move [...c, 0]
  //    >> into difference [
  //      half_plane { d: r - sag, normal: vn } >> move [...c, 0]
  //    ];
 
  return {
    ...plane,
    pos: p,
    pts: concat [plane.pts, [p]],
    stk: concat [plane.stk, [shape]],
    nrm: -[-nrm_next.[Y], nrm_next.[X]]
  };
};

const bezier4 = (points, step) => {
  const steps = 1.0 / step;
  const steps_l = (new Array(steps)).fill(0).map((e, i) => i * step);
  return steps_l.map(t =>
    (       ((1 - t) ^ 3)           * points[0])
    +  (3 * ((1 - t) ^ 2) * (t ^ 1) * points[1])
    +  (3 * ((1 - t) ^ 1) * (t ^ 2) * points[2])
    +  (                    (t ^ 3) * points[3])
  );
};

const spline = (list_of_triples_relative, plane) =>
  spline_to (list_of_triples_relative.map(t => t.map(p => p + plane.pos)), plane);

const spline_to (list_of_triples, plane) => {
  let l = list_of_triples;
  let pts = (new Array(list_of_triples.length)).fill(0).map((e, i) => {
    const p_prev = i == 0 ? plane.pos : l[i-1][2];
    return bezier4([p_prev, l[i][0], l[i][1], l[i][2]], 0.05) /* todo: calc steps */
  }).reduce((a, c) => a.concat(c), []);

  let lfst = pts[pts.length - 1];
  let lsnd = pts[pts.length - 2];
  let lln = lsnd - lfst;
  let nrm = lln / (mag lln);

  return {
    ...plane,
    pos: l.[count l - 1].[2],
    pts: concat [plane.pts, pts],
    nrm: -[nrm.[X], nrm.[Y]]
  };
};

const polyline = (list_of_tuples, plane) => {
  let p = list_of_tuples[list_of_tuples.length - 1];
  let p2 = list_of_tuples[list_of_tuples.length - 2];
  let nrm = (p-p2) / (mag (p-p2));
  return {
    ...plane,
    pos: p,
    pts: concat [plane.pts, list_of_tuples],
    nrm: nrm
  };
}

// TODO figure out what's really going to happen here when outputting
const close = (plane) => union [
  polygon (plane.pts),
  ...[for (s in plane.stk)
    s
  ]
];
