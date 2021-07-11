import * as AWS from "aws-sdk";
import { promises as fs } from "fs"
import path from "path";

AWS.config.update({ region: 'ap-northeast-1' });
const lambdaClient = new AWS.Lambda();

(async () => {
    // const result = await client.scan({ TableName }).promise();
    // console.log(result);
    const params = process.argv

    const paths = await fs.readdir("../problems");
    for (const p of paths) {
        const id = path.basename(p, ".problem");
        console.log(`Kick Problem ${id}`);

        try {
            await lambdaClient.invoke({
                FunctionName: 'Solver',
                Payload: JSON.stringify(params)
            }).promise()
            console.log(`Put Done: ${id}`);
        } catch (e) {
            // TODO: check e if unexpected exception occurs or not
            console.log(`Put Skip: ${id}`);
        }
    }
})();

