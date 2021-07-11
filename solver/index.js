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

    const MAX_MINUTES = 3; // TODO

    const result = await client.scan({
        TableName,
        ExpressionAttributeValues: {
            ":t": Date.now() - (15 * 60 * 1000),
        },
        FilterExpression: "attribute_not_exists(LastExecutionTimestamp) OR LastExecutionTimestamp < :t"
    }).promise();
    const items = result.Items;
    if (items.length === 0) {
        console.log("Nothing to update...");
        return {};
    }
    const target = items[0];
    const id = target.ProblemId;
    const problem = target.Problem;
    console.log(`Try to update ${id}, current Dislikes = ${target.Dislikes}`);

    // Update Last Execution Timestamp
    const now = Date.now();
    console.log(`Update LastExecutionTimestamp in ${id} to ${now}`);

    await client.update({
        TableName,
        Key: {
            ProblemId: id,
        },
        UpdateExpression: "set LastExecutionTimestamp = :t",
        ExpressionAttributeValues: {
            ":t": now,
            ":p": target.LastExecutionTimestamp || 0,
        },
        ConditionExpression: "attribute_not_exists(LastExecutionTimestamp) OR LastExecutionTimestamp = :p",
    }).promise();

    // Calc
    const solution = JSON.parse(child_process.execSync(SOLVER_PATH, {
        input: JSON.stringify(problem),
        env: {
            HILL_CLIMBING_TIME_LIMIT_SECONDS: `${MAX_MINUTES * 60}`,
            DISABLE_DFS_CENTROID: '1',
            RUST_BACKTRACE: '1'
        }
    }));
    const dislikes = calculateDislikes(problem, solution);
    console.log(`Dislikes = ${dislikes}, solution = ${JSON.stringify(solution)}`);

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
