import * as AWS from "aws-sdk";
import fetch from "node-fetch";

type Point = [number, number];

interface Solution {
    vertices: Point[];
}

const API_KEY = process.env.API_KEY;

AWS.config.update({ region: 'ap-northeast-1' })
const client = new AWS.DynamoDB.DocumentClient();
const ProblemsTableName = 'Problems';
const SolutionsTableName = 'Solutions';

(async () => {
    let ProblemsExclusiveStartKey = undefined;

    do {
        const result: any = await client.scan({
            TableName: ProblemsTableName,
            ExclusiveStartKey: ProblemsExclusiveStartKey,
        }).promise()
        for (const item of result.Items) {
            let solutions: any[] = []
            let SolutionsExclusiveStartKey = undefined
            do {
                const solutionsResult: any = await client.scan({
                    TableName: SolutionsTableName,
                    ExclusiveStartKey: SolutionsExclusiveStartKey,
                    FilterExpression: "ProblemId = :id AND (attribute_not_exists(Pose.bonuses) OR Pose.bonuses = :bonuses)",
                    ExpressionAttributeValues: {
                        ":id": item.ProblemId,
                        ":bonuses": [], // No bonus
                    }
                }).promise()
                solutions = solutions.concat(solutionsResult.Items)
                SolutionsExclusiveStartKey = solutionsResult.LastEvaluatedKey
            } while (SolutionsExclusiveStartKey)

            if (solutions.length === 0) {
                console.log(`No solutions: ProblemId = ${item.ProblemId}`)
                continue;
            }
            solutions.sort((a, b) => a.Dislikes - b.Dislikes)
            const solution = solutions[0].Pose
            if (!solution.bonuses) {
                solution.bonuses = []
            }
            console.log(`Solution: ProblemId = ${item.ProblemId}, Solution = ${JSON.stringify(solution)}`)
            try {
                const res = await fetch(`https://poses.live/api/problems/${item.ProblemId}/solutions`, {
                    method: 'POST',
                    headers: {
                        Authorization: `Bearer ${API_KEY}`,
                        'Content-Type': 'applicationb/json',
                    },
                    body: JSON.stringify(solution),
                });
                console.log(await res.json());
            } catch (e) {
                // console.log(e);
            }
            // process.exit(0);
        }
        ProblemsExclusiveStartKey = result.LastEvaluatedKey;
    } while (ProblemsExclusiveStartKey)

})();
