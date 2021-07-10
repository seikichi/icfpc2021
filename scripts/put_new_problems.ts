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

(async () => {
    // const result = await client.scan({ TableName }).promise();
    // console.log(result);
    const paths = await fs.readdir("../problems");
    for (const p of paths) {
        const id = path.basename(p, ".problem");
        const problem = JSON.parse(await fs.readFile(path.join('..', 'problems', p), 'utf-8')) as Problem;

        const NumEdges = problem.figure.edges.length;
        const NumVertices = problem.figure.vertices.length;
        const NumHole = problem.hole.length;

        console.log(`Put Problem ${id}`);

        const Item = {
            ProblemId: id,
            Problem: problem,
            NumEdges,
            NumVertices,
            NumHole
        };

        try {
            await client.put({
                TableName,
                Item,
                ConditionExpression: "attribute_not_exists(ProblemId)",
            }).promise();
            console.log(`Put Done: ${id}`);
        } catch (e) {
            // TODO: check e if unexpected exception occurs or not
            console.log(`Put Skip: ${id}`);
        }
    }
})();

