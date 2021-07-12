const AWS = require("aws-sdk");
const child_process = require('child_process');

AWS.config.update({
    region: 'ap-northeast-1'
})
const client = new AWS.DynamoDB.DocumentClient();
const ProblemsTableName = 'Problems';
const SolutionsTableName = 'Solutions';

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
    const commitHash = process.env.COMMIT
    const ProblemId = `${event.problemId}`
    const params = { MILP: '1', ...event.env }
    const commitAndParams = `${commitHash}:${Object.keys(params).sort().map(key => `${key}=${params[key]}`).join("&")}`

    const SOLVER_PATH = './new_solver.py';

    const item = await client.get({
        TableName: ProblemsTableName,
        Key: {
            ProblemId
        }
    }).promise()
    const problem = item.Item.Problem;

    // Calc
    console.log(`Start: ProblemId = ${ProblemId}, CommitHash = ${commitHash}, Params = ${JSON.stringify(params)}`)
    const solution = JSON.parse(child_process.execSync(SOLVER_PATH, {
        input: JSON.stringify(problem),
        env: {
            ...event.env,
        }
    }));
    const dislikes = calculateDislikes(problem, solution);
    console.log(`Dislikes = ${dislikes}, solution = ${JSON.stringify(solution)}`);

    const paramsForPut = {
        TableName: SolutionsTableName,
        Item: {
            ProblemId,
            "Commit:Params": commitAndParams,
            Dislikes: dislikes,
            Pose: solution,
            UnlockBonuses: [], // TODO
        },
        ConditionExpression: "attribute_not_exists(ProblemId)",
    }

    await client.put(paramsForPut).promise();
    console.log(`Solution Put Done ${ProblemId}`);
    return {}
}
