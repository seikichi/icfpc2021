import * as AWS from "aws-sdk";
const Confirm: any = require('prompt-confirm')

AWS.config.update({ region: 'ap-northeast-1' });
const lambdaClient = new AWS.Lambda();
const dynamoClient = new AWS.DynamoDB.DocumentClient();
const TableName = "Problems";

(async () => {
    const env: any = {}

    if (process.argv.length >= 3) {
        process.argv[2].split("&").map(p => {
            const param = p.split("=")
            env[param[0]] = param[1]
        })
    }

    const prompt = new Confirm(`Are you sure to kick lambda for all the problems? : env = ${JSON.stringify(env)}`)
    const confirm = await prompt.run()
    if (!confirm) {
        console.log("Aborted")
        return
    }

    const problems = (await dynamoClient.scan({ TableName }).promise()).Items!
    for (const p of problems) {
        const id = p.ProblemId

        const params: any = {
            problemId: id,
            env
        }

        console.log(`Lamdba Start Problem ${id}: params = ${JSON.stringify(params)}`);
        try {
            await lambdaClient.invoke({
                FunctionName: 'AutomationStack-Solver4A42070C-fAxknDnlrUfm',
                InvocationType: "Event",
                Payload: JSON.stringify(params)
            }).promise()
            console.log(`Lambda Kicked: ${id}`);
        } catch (e) {
            throw e
        }
    }
})();

