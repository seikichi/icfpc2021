import re
import json
import sys
import math
from functools import cache
from collections import defaultdict

from pyscipopt import Model
from shapely.geometry import Point
from shapely.geometry.polygon import Polygon, LineString

MAX_D_FROM_HOLE = 1000


def distance(x1, y1, x2, y2):
    return (x1 - x2) ** 2 + (y1 - y2) ** 2


problem_id = sys.argv[1]

# INPUT = "../problems/14.problem"
INPUT = f"../problems/{problem_id}.problem"

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

epsilon = problem['epsilon']
vertices = problem['figure']['vertices']
edges = problem['figure']['edges']

print(f"# of vertices = {len(vertices)}")
print(f"# of hole = {len(problem['hole'])}")
print(f"# of candidates = {len(candidates)}")

rings_by_distance = defaultdict(list)

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

distance_candidates_by_hole = []

for i in range(len(problem['hole'])):
    hx, hy = problem['hole'][i]
    ds = set()
    for (cx, cy) in candidates:
        d = distance(hx, hy, cx, cy)
        if d <= MAX_D_FROM_HOLE:
            ds.add(d)
    distance_candidates_by_hole.append(ds)

model = Model("icfpc2021")

b = {}
l = {}

print("create variables (b) ...")
for i in range(len(vertices)):
    for (cx, cy) in candidates:
        b[(i, cx, cy)] = model.addVar(vtype='B')

print("create variables (l) ...")
for i in range(len(problem['hole'])):
    for d in distance_candidates_by_hole[i]:
        l[(i, d)] = model.addVar(vtype="C", lb=0, ub=1)


print("add sum b == 1 constraints ...")
for i in range(len(vertices)):
    model.addCons(sum(b[(i, cx, cy)] for (cx, cy) in candidates) == 1)

print("add sum l == 1 constraints ...")
for i in range(len(problem['hole'])):
    model.addCons(sum(l[(i, d)] for d in distance_candidates_by_hole[i]) == 1)

print("add l <= sum b constraints ...")
for i in range(len(problem['hole'])):
    hx, hy = problem['hole'][i]
    dic = defaultdict(list)

    for (cx, cy) in candidates:
        d = distance(hx, hy, cx, cy)
        dic[d].append((cx, cy))

    for d in distance_candidates_by_hole[i]:
        model.addCons(l[(i, d)] <= sum(b[(j, cx, cy)]
                      for j in range(len(vertices)) for (cx, cy) in dic[d]))

print("add b <= sum b constraints for edges ...")
for p, (vi, wi) in enumerate(edges):
    print(f"{p} / {len(edges)} ...")
    vx, vy = vertices[vi]
    wx, wy = vertices[wi]
    d = distance(vx, vy, wx, wy)

    c = (1.0 * d * epsilon) / 1e6
    lb = math.ceil(d - c)
    ub = math.floor(d + c)

    for (cx, cy) in candidates:
        points = []
        for n in range(lb, ub + 1):
            for (dx, dy) in points_by_distances[n]:
                p = Point(cx + dx, cy + dy)
                if not (hole.contains(p) or hole.boundary.contains(p)):
                    continue
                # Is the following condition correct?
                if LineString([(cx, cy), (cx + dx, cy + dy)]).crosses(hole):
                    continue
                points.append((cx + dx, cy + dy))
        model.addCons(b[(vi, cx, cy)] <= sum(b[(wi, nx, ny)]
                      for (nx, ny) in points))
        model.addCons(b[(wi, cx, cy)] <= sum(b[(vi, nx, ny)]
                      for (nx, ny) in points))


print("set objective ...")
model.setObjective(sum(
    d * l[(i, d)]
    for i in range(len(problem['hole']))
    for d in distance_candidates_by_hole[i]
), "minimize")


print("Optimize Start!")
model.optimize()

print("Optimal value:", model.getObjVal())

vertices = []
for i in range(len(problem['figure']['vertices'])):
    for (cx, cy) in candidates:
        if model.getVal(b[(i, cx, cy)]) > 0.0:
            vx, vy = problem['figure']['vertices'][i]
            print(f"({vx}, {vy}) -> ({cx}, {cy})")
            vertices.append([cx, cy])
            break


with open(f"solutions/{problem_id}.solution", 'w', encoding='utf-8') as f:
    json.dump({'vertices': vertices}, f)
