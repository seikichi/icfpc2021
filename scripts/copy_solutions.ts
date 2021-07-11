import * as AWS from "aws-sdk";

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
const ProblemsTableName = 'Problems';
const SolutionsTableName = 'Solutions';


(async () => {
    let result = await client.scan({ TableName: ProblemsTableName }).promise()
    let items = result.Items!
    while (result.LastEvaluatedKey !== undefined) {
        result = await client.scan({
            TableName: ProblemsTableName,
            ExclusiveStartKey: result.LastEvaluatedKey
        }).promise()
        items = items.concat(result.Items!)
    }

    const initialCommitParams = "071685d:"

    for (const p of items) {
        const id = p.ProblemId
        const Item = {
            ProblemId: id,
            "Commit:Params": initialCommitParams,
            Dislikes: p.Dislikes,
            Pose: p.Solution,
            UsedBonuses: [],
            UnlockBonuses: [],
        };

        try {
            await client.put({
                TableName: SolutionsTableName,
                Item,
                ConditionExpression: "attribute_not_exists(ProblemId)",
            }).promise();
            console.log(`Put Done: ${id}`);
        } catch (e) {
            // TODO: check e if unexpected exception occurs or not
            console.log(`Put Skip: ${id}`);
        }
    }

    const sol = await client.scan({
        TableName: SolutionsTableName
    }).promise();
    console.log(sol.Items!.length)
})();
