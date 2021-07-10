import * as AWS from "aws-sdk";
import fetch from "node-fetch";

type Point = [number, number];

interface Solution {
    vertices: Point[];
}

const API_KEY = process.env.API_KEY;

AWS.config.update({ region: 'ap-northeast-1' })
const client = new AWS.DynamoDB.DocumentClient();
const TableName = 'Problems';

(async () => {
    let ExclusiveStartKey = undefined;

    do {
        const result: any = await client.scan({
            TableName,
            ExclusiveStartKey,
        }).promise()
        for (const item of result.Items) {
            if (!item.Solution) {
                continue;
            }
            try {
                const res = await fetch(`https://poses.live/api/problems/${item.ProblemId}/solutions`, {
                    method: 'POST',
                    headers: {
                        Authorization: `Bearer ${API_KEY}`,
                        'Content-Type': 'applicationb/json',
                    },
                    body: JSON.stringify(item.Solution),
                });
                console.log(await res.json());
            } catch (e) {
                // console.log(e);
            }
            // process.exit(0);
        }
        ExclusiveStartKey = result.LastEvaluatedKey;
    } while (ExclusiveStartKey)

})();
