import * as AWS from "aws-sdk";

type Point = [number, number];

interface Solution {
    bonuses: any[];
    vertices: Point[];
}


AWS.config.update({ region: 'ap-northeast-1' })
const client = new AWS.DynamoDB.DocumentClient();
const SolutionsTableName = 'Solutions';

const solution: Solution = {
    "bonuses": [],
    "vertices": [[5, 77], [0, 76], [7, 68], [21, 62], [11, 77], [20, 44], [20, 30], [20, 28], [29, 68], [23, 36], [18, 34], [16, 70], [17, 21], [10, 25], [10, 27], [21, 64], [20, 29], [16, 26], [23, 77], [17, 58], [15, 31], [20, 74], [27, 29], [24, 47], [57, 20], [59, 20], [46, 17], [24, 31], [54, 17], [56, 1], [56, 3], [50, 15], [54, 17], [55, 4], [52, 11], [51, 0], [48, 10], [33, 23], [31, 12]]
};

(async () => {
    const initialCommitParams = "DUMMY:MILP=1"

    const id = '73';
    const Item = {
        ProblemId: id,
        "Commit:Params": initialCommitParams,
        Dislikes: 0,
        Pose: solution,
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
})();
