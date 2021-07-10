import * as AWS from "aws-sdk";
import { promises as fs } from "fs"
import path from "path";

type Point = [number, number];

interface Problem {
    hole: Point[]
    figure: {
        vertices: Point[];
        edges: [number, number][];
    };
    epsilon: number;
}

interface Solution {
    vertices: Point[];
}


AWS.config.update({ region: 'ap-northeast-1' })
const client = new AWS.DynamoDB.DocumentClient();
const TableName = 'Problems';

function distance(p1: Point, p2: Point): number {
    return (p1[0] - p2[0]) ** 2 + (p1[1] - p2[1]) ** 2;
}

function calculateDislikes(problem: Problem, solution: Solution): number {
    let dislikes = 0;
    for (let h of problem.hole) {
        let min = Infinity;
        for (let p of solution.vertices) {
            min = Math.min(min, distance(h, p));
        }
        dislikes += min;
    }
    return dislikes;
}

(async () => {
    // const result = await client.scan({ TableName }).promise();
    // console.log(result);
    const paths = await fs.readdir("../solutions");
    for (const p of paths) {
        const id = path.basename(p, ".solution");
        const problem = JSON.parse(await fs.readFile(path.join('..', 'problems', `${id}.problem`), 'utf-8')) as Problem;
        const solution = JSON.parse(await fs.readFile(path.join('..', 'solutions', `${id}.solution`), 'utf-8')) as Solution;

        // const NumEdges = problem.figure.edges.length;
        // const NumVertices = problem.figure.vertices.length;
        // const NumHole = problem.hole.length;
        const dislikes = calculateDislikes(problem, solution);

        try {
            await client.update({
                TableName,
                Key: {
                    ProblemId: id,
                },
                UpdateExpression: "set Dislikes = :d, Solution = :s",
                ExpressionAttributeValues: {
                    ":d": dislikes,
                    ":s": solution,
                },
                ConditionExpression: "attribute_not_exists(Dislikes) OR Dislikes > :d",
            }).promise();

            console.log(`Solution Update Done ${id}`);
        } catch (e) {
            if (e.code !== "ConditionalCheckFailedException") {
                throw e;
            }
            console.log(`Solution Update Skip ${id}`);

        }
    }
})();

