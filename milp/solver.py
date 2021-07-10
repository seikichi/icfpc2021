import re
import json
import sys

from pyscipopt import Model
from shapely.geometry import Point
from shapely.geometry.polygon import Polygon

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
    # if hole.boundary.contains(Point(x, y))
]
print(f"# of candidates = {len(candidates)}")

model = Model("icfpc2021")

b = {}
n = {}

print("create variables (b) ...")
for i in range(len(problem['figure']['vertices'])):
    for (cx, cy) in candidates:
        b[(i, cx, cy)] = model.addVar(vtype='B')

print("create variables (n) ...")
for (hx, hy) in problem['hole']:
    for (cx, cy) in candidates:
        # n[(hx, hy, cx, cy)] = model.addVar(vtype='I')
        n[(hx, hy, cx, cy)] = model.addVar(vtype='C', lb=0, ub=1)

print("add sum b == 1 constraints ...")
for i in range(len(problem['figure']['vertices'])):
    model.addCons(sum(b[(i, cx, cy)] for (cx, cy) in candidates) == 1)

print("add sum n == 1 constraints ...")
for (hx, hy) in problem['hole']:
    model.addCons(sum(n[(hx, hy, cx, cy)] for (cx, cy) in candidates) == 1)

print("add n <= sum b constraints ...")
for (hx, hy) in problem['hole']:
    for (cx, cy) in candidates:
        model.addCons(n[(hx, hy, cx, cy)] <= sum(b[(i, cx, cy)]
                      for i in range(len(problem['figure']['vertices']))))


def d(x1, y1, x2, y2):
    return (x1 - x2) ** 2 + (y1 - y2) ** 2


# TODO: need to shrink...
print("add b_i + b_j <= 1 constraints ...")
epsilon = problem['epsilon']
for (vi, wi) in problem['figure']['edges']:
    vx, vy = problem['figure']['vertices'][vi]
    wx, wy = problem['figure']['vertices'][wi]
    for (cx1, cy1) in candidates:
        for (cx2, cy2) in candidates:
            d_before = d(vx, vy, wx, wy)
            d_after = d(cx1, cy1, cx2, cy2)
            if abs(d_after - d_before) * 1e6 > epsilon * d_before:
                model.addCons(b[(vi, cx1, cy1)] +
                              b[(wi, cx2, cy2)] <= 1)

print("set objective ...")
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
for i in range(len(problem['figure']['vertices'])):
    for (cx, cy) in candidates:
        if model.getVal(b[(i, cx, cy)]) > 0.0:
            vx, vy = problem['figure']['vertices'][i]
            print(f"({vx}, {vy}) -> ({cx}, {cy})")
            vertices.append([cx, cy])
            break


with open(f"solutions/{problem_id}.solution", 'w', encoding='utf-8') as f:
    json.dump({'vertices': vertices}, f)
