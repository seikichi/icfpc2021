#!/usr/bin/env python3

import json
import sys
import math
from collections import defaultdict

from pyscipopt import Model
from shapely.geometry import Point
from shapely.geometry.polygon import Polygon, LineString, LinearRing


def distance(x1, y1, x2, y2):
    return (x1 - x2) ** 2 + (y1 - y2) ** 2


problem = json.load(sys.stdin)

hole = problem['hole']
epsilon = problem['epsilon']
vertices = problem['figure']['vertices']
edges = problem['figure']['edges']

print(f"# of vertices = {len(vertices)}", file=sys.stderr)
print(f"# of edges = {len(edges)}", file=sys.stderr)
print(f"# of hole = {len(hole)}", file=sys.stderr)


def find_largest_sub_convex_hole(hole):
    max_size = -1
    result = None
    for start in range(len(hole)):
        prev = hole[start]
        curr = hole[(start + 1) % len(hole)]
        points = [prev, curr]

        for i in range(2, len(hole)):
            n = hole[(start + i) % len(hole)]
            ring = LinearRing([prev, curr, n])
            if not ring.is_ccw:
                continue
            prev, curr = curr, n
            points.append(n)

        if not LinearRing([prev, curr, hole[start]]).is_ccw:
            continue

        size = Polygon(points).area
        if max_size < size:
            max_size = size
            result = points

    return result


convex_hole = find_largest_sub_convex_hole(hole)
# convex_hole = hole  # FIXME

hole_ploy = Polygon(hole)
convex_hole_poly = Polygon(convex_hole)

xmin, ymin, xmax, ymax = map(int, hole_ploy.bounds)
print(f"xmin = {xmin}, ymin = {ymin}, xmax = {xmax}, ymax = {ymax}",
      file=sys.stderr)
print(f"hole = ${hole}", file=sys.stderr)
print(f"convex hole = ${convex_hole}", file=sys.stderr)

points_by_distances = {}

for (vi, wi) in edges:
    vx, vy = vertices[vi]
    wx, wy = vertices[wi]
    d = distance(vx, vy, wx, wy)

    c = (1.0 * d * epsilon) / 1e6
    lb = math.ceil(d - c)
    ub = math.floor(d + c)
    for n in range(lb, ub + 1):
        points_by_distances[n] = []

for i in range(-xmax, xmax + 1):
    for j in range(-ymax, ymax + 1):
        d = distance(0, 0, i, j)
        if d in points_by_distances:
            points_by_distances[d].append((i, j))

model = Model("icfpc2021")
model.hideOutput()
# model.setRealParam('limits/time', 10)  # FIXME

# MAX_D = max(xmax - xmin + 1, ymax - ymin + 1)  # TODO
MAX_D = max(1, int(max(xmax - xmin + 1, ymax - ymin + 1) / 4))

print("create variables (b) ...", file=sys.stderr)
pvx = []
pvy = []
for i in range(len(vertices)):
    pvx.append(model.addVar(vtype='I', lb=xmin, ub=xmax))
    pvy.append(model.addVar(vtype='I', lb=ymin, ub=ymax))

dx_candidates_of_edge = []
dy_candidates_of_edge = []

for i, (vi, wi) in enumerate(edges):
    vx, vy = vertices[vi]
    wx, wy = vertices[wi]
    d = distance(vx, vy, wx, wy)

    c = (1.0 * d * epsilon) / 1e6
    lb = math.ceil(d - c)
    ub = math.floor(d + c)

    dxs = set()
    dys = set()
    for n in range(lb, ub + 1):
        for (dx, dy) in points_by_distances[n]:
            dxs.add(dx)
            dys.add(dy)
    dx_candidates_of_edge.append(dxs)
    dy_candidates_of_edge.append(dys)

print("create edge length variable (el) ...", file=sys.stderr)
elx = []
ely = []
for i in range(len(edges)):
    elx.append({dx: model.addVar(vtype="B")
               for dx in dx_candidates_of_edge[i]})
    ely.append({dy: model.addVar(vtype="B")
               for dy in dy_candidates_of_edge[i]})

print("create edge length constraints ...", file=sys.stderr)
for i, (vi, wi) in enumerate(edges):
    model.addCons(sum(elx[i][dx] for dx in dx_candidates_of_edge[i]) == 1)
    model.addCons(sum(ely[i][dy] for dy in dy_candidates_of_edge[i]) == 1)

    model.addCons((pvx[vi] - pvx[wi]) == sum(dx * elx[i][dx]
                  for dx in dx_candidates_of_edge[i]))
    model.addCons((pvy[vi] - pvy[wi]) == sum(dy * ely[i][dy]
                  for dy in dy_candidates_of_edge[i]))

    vx, vy = vertices[vi]
    wx, wy = vertices[wi]
    d = distance(vx, vy, wx, wy)
    c = (1.0 * d * epsilon) / 1e6

    model.addCons(
        (sum(dx * dx * elx[i][dx] for dx in dx_candidates_of_edge[i]) +
         sum(dy * dy * ely[i][dy] for dy in dy_candidates_of_edge[i])) <= d + c
    )
    model.addCons(
        (sum(dx * dx * elx[i][dx] for dx in dx_candidates_of_edge[i]) +
         sum(dy * dy * ely[i][dy] for dy in dy_candidates_of_edge[i])) >= d - c
    )

for j in range(len(vertices)):
    for i in range(len(convex_hole)):
        h1 = convex_hole[i]
        h2 = convex_hole[(i + 1) % len(convex_hole)]
        vx, vy = h1
        wx, wy = h2

        cx = (wx - vx)
        cy = (wy - vy)

        # CCW (あやしい)
        model.addCons(cx * (pvy[j] - vy) - cy * (pvx[j] - vx) >= 0)

sign = []
for i in range(len(hole)):
    sign.append([])
    for j in range(len(vertices)):
        sign[-1].append(model.addVar(vtype="B"))

for i in range(len(hole)):
    model.addCons(sum(sign[i][j] for j in range(len(vertices))) == 1)

hdlx = []
hdly = []
for i in range(len(hole)):
    hdlx.append([])
    hdly.append([])
    for j in range(MAX_D):
        hdlx[-1].append(model.addVar(vtype="B"))
        hdly[-1].append(model.addVar(vtype="B"))

for i in range(len(hole)):
    model.addCons(sum(hdlx[i][j] for j in range(MAX_D)) == 1)
    model.addCons(sum(hdly[i][j] for j in range(MAX_D)) == 1)

M = 1000000

for i in range(len(hole)):
    hx, hy = hole[i]
    for k in range(len(vertices)):
        model.addCons(
            pvx[k] - hx <= sum(j * hdlx[i][j]
                               for j in range(MAX_D)) + M * (1 - sign[i][k])

        )
        model.addCons(
            pvx[k] - hx >= -sum(j * hdlx[i][j]
                                for j in range(MAX_D)) - M * (1 - sign[i][k])
        )
        model.addCons(
            pvy[k] - hy <= sum(j * hdly[i][j]
                               for j in range(MAX_D)) + M * (1 - sign[i][k])
        )
        model.addCons(
            pvy[k] - hy >= -sum(j * hdly[i][j]
                                for j in range(MAX_D)) - M * (1 - sign[i][k])
        )

model.setObjective(
    sum(
        j * j * hdlx[i][j] + j * j * hdly[i][j]
        for i in range(len(hole))
        for j in range(MAX_D)
    ),
    "minimize"
)
model.optimize()

print("Optimal value:", model.getObjVal(), file=sys.stderr)
print("Gap:", model.getGap(), file=sys.stderr)

result = []
for i in range(len(vertices)):
    vx, vy = vertices[i]
    nx = model.getVal(pvx[i])
    ny = model.getVal(pvy[i])
    print(f"({nx}, {ny}) <- ({vx}, {vy})", file=sys.stderr)
    result.append([int(round(nx)), int(round(ny))])

# for i, (vi, wi) in enumerate(edges):
#     print(f"edge {i}: {vi} -> {wi}")
#     for dx in dx_candidates_of_edge[i]:
#         if model.getVal(elx[i][dx]) > 0.5:
#             print(f"elx[{i}][{dx}] = {model.getVal(elx[i][dx])}")
#             print(f"pvx[{vi}] = {model.getVal(pvx[vi])}")
#             print(f"pvx[{wi}] = {model.getVal(pvx[wi])}")
#     for dy in dy_candidates_of_edge[i]:
#         if model.getVal(ely[i][dy]) > 0.5:
#             print(f"ely[{i}][{dy}] = {model.getVal(ely[i][dy])}")
#             print(f"pvy[{vi}] = {model.getVal(pvy[vi])}")
#             print(f"pvy[{wi}] = {model.getVal(pvy[wi])}")

print(json.dumps({'vertices': result}))
