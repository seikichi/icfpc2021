import * as cdk from '@aws-cdk/core';
import * as lambda from "@aws-cdk/aws-lambda";
import * as dynamodb from "@aws-cdk/aws-dynamodb";
// import * as events from "@aws-cdk/aws-events";
// import * as targets from "@aws-cdk/aws-events-targets";

export class AutomationStack extends cdk.Stack {
  constructor(scope: cdk.Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const fun = new lambda.DockerImageFunction(this, "Solver", {
      code: lambda.DockerImageCode.fromImageAsset("../solver"),
      timeout: cdk.Duration.minutes(15),
      memorySize: 512,
    });

    // DB
    const STRING = dynamodb.AttributeType.STRING;

    const problems = new dynamodb.Table(this, "Problems", {
      tableName: "Problems",
      partitionKey: { name: "ProblemId", type: STRING },
    });

    const solutions = new dynamodb.Table(this, "Solutions", {
      tableName: "Solutions",
      partitionKey: { name: "ProblemId", type: STRING },
      sortKey: {
        name: "Commit:Params", // xyz123:BAR=1&FOO=1
        type: STRING,
      }
    });

    problems.grantReadWriteData(fun); // TODO: Change to grantReadData
    solutions.grantReadWriteData(fun);
  }
}
