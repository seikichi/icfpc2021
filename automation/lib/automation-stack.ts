import * as cdk from '@aws-cdk/core';
import * as lambda from "@aws-cdk/aws-lambda";
import * as dynamodb from "@aws-cdk/aws-dynamodb";
// import * as events from "@aws-cdk/aws-events";
// import * as targets from "@aws-cdk/aws-events-targets";

import * as child_process from "child_process"

export class AutomationStack extends cdk.Stack {
  constructor(scope: cdk.Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    // get commit Hash
    const commitHash = child_process.execSync("git rev-parse --short HEAD").toString().trim()

    const fun = new lambda.DockerImageFunction(this, "Solver", {
      code: lambda.DockerImageCode.fromImageAsset("../solver"),
      timeout: cdk.Duration.minutes(15),
      memorySize: 512,
      environment: { COMMIT: commitHash }
    });

    const milp = new lambda.DockerImageFunction(this, "MILP", {
      code: lambda.DockerImageCode.fromImageAsset("../milp"),
      timeout: cdk.Duration.minutes(15),
      memorySize: 2048,
      environment: { COMMIT: commitHash }
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

    problems.grantReadWriteData(fun);
    solutions.grantReadWriteData(fun);

    problems.grantReadWriteData(milp);
    solutions.grantReadWriteData(milp);
  }
}
