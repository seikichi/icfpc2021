import re
import json

from pyscipopt import Model
from shapely.geometry import Point
from shapely.geometry.polygon import Polygon

INPUT = "../problems/14.problem"

with open(INPUT, encoding='utf-8') as f:
    problem = json.load(f)

hole = Polygon(problem['hole'])

xmin, ymin, xmax, ymax = map(int, hole.bounds)
print(f"xmin = {xmin}, ymin = {ymin}, xmax = {xmax}, ymax = {ymax}")

candidates = [
    (x, y)
    for x in range(xmin, xmax + 1)
    for y in range(ymin, ymax + 1)
    if hole.contains(Point(x, y)) or hole.boundary.contains(Point(x, y))
]
print(f"# of candidates = {len(candidates)}")

model = Model("icfpc2021")

b = {}
n = {}

for (vx, vy) in problem['figure']['vertices']:
    for (cx, cy) in candidates:
        b[(vx, vy, cx, cy)] = model.addVar(vtype='I')

for (hx, hy) in problem['hole']:
    for (cx, cy) in candidates:
        n[(hx, hy, cx, cy)] = model.addVar(vtype='I')

for (vx, vy) in problem['figure']['vertices']:
    model.addCons(sum(b[(vx, vy, cx, cy)] for (cx, cy) in candidates) == 1)

for (hx, hy) in problem['hole']:
    model.addCons(sum(n[(hx, hy, cx, cy)] for (cx, cy) in candidates) == 1)

for (hx, hy) in problem['hole']:
    for (cx, cy) in candidates:
        model.addCons(n[(hx, hy, cx, cy)] <= sum(b[(vx, vy, cx, cy)]
                      for (vx, vy) in problem['figure']['vertices']))


def d(x1, y1, x2, y2):
    return (x1 - x2) ** 2 + (y1 - y2) ** 2


# TODO: need to shrink...
epsilon = problem['epsilon']
for (vi, wi) in problem['figure']['edges']:
    vx, vy = problem['figure']['vertices'][vi]
    wx, wy = problem['figure']['vertices'][wi]
    for (cx1, cy1) in candidates:
        for (cx2, cy2) in candidates:
            d_before = d(vx, vy, wx, wy)
            d_after = d(cx1, cy1, cx2, cy2)
            if abs(d_after - d_before) * 1e6 > epsilon * d_before:
                model.addCons(b[(vx, vy, cx1, cy1)] +
                              b[(wx, wy, cx2, cy2)] <= 1)

model.setObjective(sum(
    d(hx, hy, cx, cy) * n[(hx, hy, cx, cy)]
    for (hx, hy) in problem['hole']
    for (cx, cy) in candidates
), "minimize")

print("Optimize Start!")

model.optimize()

print("Optimal value:", model.getObjVal())

# print(hole.contains(p))
# print(hole.boundary.contains(p))

vertices = []
for (vx, vy) in problem['figure']['vertices']:
    for (cx, cy) in candidates:
        if model.getVal(b[(vx, vy, cx, cy)]) > 0.0:
            vertices.append([cx, cy])
            break


with open('out.solution', 'w', encoding='utf-8') as f:
    json.dump({'vertices': vertices}, f)
