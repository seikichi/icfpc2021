const AWS = require("aws-sdk");
const child_process = require('child_process');

AWS.config.update({
    region: 'ap-northeast-1'
})
const client = new AWS.DynamoDB.DocumentClient();
const TableName = 'Problems';

function distance(p1, p2) {
    return (p1[0] - p2[0]) ** 2 + (p1[1] - p2[1]) ** 2;
}

function calculateDislikes(problem, solution) {
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

exports.handler = async function (event, context) {
    const SOLVER_PATH = '/code/target/release/icfpc2021';
    const id = "1"
    const item = await client.get({ TableName, Key: { ProblemId: '1' } }).promise();

    const problem = item.Item.Problem;
    const solution = JSON.parse(child_process.execSync(SOLVER_PATH, { input: JSON.stringify(problem) }));

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

    return {}
}
