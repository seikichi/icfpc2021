import { useState, useRef, createRef, useEffect, RefObject, Fragment } from "react";
import { makeStyles } from '@material-ui/core/styles';
import Table from '@material-ui/core/Table';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableContainer from '@material-ui/core/TableContainer';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';
import Paper from '@material-ui/core/Paper';
import Collapse from '@material-ui/core/Collapse';
import IconButton from '@material-ui/core/IconButton';
import Box from '@material-ui/core/Box';
import Typography from '@material-ui/core/Typography';
import KeyboardArrowDownIcon from '@material-ui/icons/KeyboardArrowDown';
import KeyboardArrowUpIcon from '@material-ui/icons/KeyboardArrowUp';
import { JSDOM } from "jsdom"

import AWS from "aws-sdk"

type Point = [number, number];

interface Problem {
  hole: Point[]
  figure: {
    vertices: Point[];
    edges: [number, number][];
  };
  epsilon: number;
  [bonuses: number]: {
    position: Point,
    bonus: string,
    problem: number,
  }
}

interface Pose {
  [bonuses: number]: {
    bonus: string,
    problem: number,
  },
  vertices: Point[];
}

interface Solution {
  ProblemId: string,
  "Commit:Params": string,
  Dislikes: number,
  Pose: Pose,
  Score: number,
}

const useStyles = makeStyles({
  table: {
    minWidth: 650,
  },
});

interface Props {
  rows: readonly TableRowData[];
  rowsPerParams: readonly TableRowDataForParams[];
}

interface TableRowDataForParams {
  commitHash: string;
  params: string;
  solved: number;
  score: number;
  allSolutions: Solution[];
}

interface TableRowData {
  id: number;
  dislike: number;
  minimalDislike: number;
  commitHash: string;
  params: string;
  score: number;
  problem: Problem;
  solution: Solution | null;
  allSolutions: Solution[];
}

export async function getStaticProps() {
  AWS.config.update({ region: 'ap-northeast-1' })
  const docClient = new AWS.DynamoDB.DocumentClient()
  const TableName = "Problems"
  const SolutionsTableName = 'Solutions'
  const SESSION_ID = process.env.SESSION_ID;

  // Scan all Solutions
  let solutions: any[] = []
  let SolutionsExclusiveStartKey = undefined
  do {
    const solutionRespose: any = await docClient.scan({
      TableName: SolutionsTableName,
      ExclusiveStartKey: SolutionsExclusiveStartKey,
    }).promise()
    solutions = solutions.concat(solutionRespose.Items)
    SolutionsExclusiveStartKey = solutionRespose.LastEvaluatedKey
  } while (SolutionsExclusiveStartKey)

  // Scan all problems
  let ExclusiveStartKey = undefined
  let problems: any[] = []
  do {
    const response = await docClient.scan({ TableName }).promise()
    problems = problems.concat(response.Items)
    ExclusiveStartKey = response.LastEvaluatedKey
  } while (ExclusiveStartKey)

  // Fetch minimal dislikes
  let minimalDislikesRows: any;
  try {
    const res = await fetch(`https://poses.live/problems`, {
      method: 'GET',
      headers: {
        cookie: `session=${SESSION_ID};`
      },
    })
    const dom = new JSDOM(await res.text())
    minimalDislikesRows = dom.window.document.getElementsByTagName("table")[0].rows;
  } catch (e) {
    console.log(e);
  }

  // calculate scores for all solutions
  for (const s of solutions) {
    const problem = problems.find(a => a.ProblemId === s.ProblemId)
    const minimalDislike = parseInt(minimalDislikesRows[parseInt(problem.ProblemId)].cells[2].textContent)
    const numVertices = problem.NumVertices
    const numEdges = problem.NumEdges
    const numHole = problem.NumHole
    const score = s ? Math.ceil(1000 * Math.log2((numVertices * numEdges * numHole) / 6.0) * Math.sqrt((1.0 * minimalDislike + 1) / (s.Dislikes + 1))) : 0
    s.score = score
  }

  const solutionsForEachParams: any = {}
  const commitParams = Array.from(new Set(solutions.map((s) => s["Commit:Params"])))

  for (const cp of commitParams) {
    solutionsForEachParams[cp] = solutions.filter((s) => s["Commit:Params"] == cp)
    const totalScoreForEachParams = solutionsForEachParams[cp].reduce((a: any, b: any) => a + b.score, 0)
  }

  const rows = problems.sort((a, b) => parseInt(a.ProblemId) - parseInt(b.ProblemId)).map((item): TableRowData => {
    const filtered = solutions.filter(s => s.ProblemId === item.ProblemId)
    filtered.sort((a, b) => a.Dislikes - b.Dislikes)
    const solution = filtered[0]
    const minimalDislike = parseInt(minimalDislikesRows[parseInt(item.ProblemId)].cells[2].textContent)
    return {
      id: parseInt(item.ProblemId),
      dislike: solution ? solution.Dislikes : -1,
      problem: item.Problem as Problem,
      commitHash: solution ? solution["Commit:Params"].split(":")[0] : "",
      params: solution ? solution["Commit:Params"].split(":")[1] : "",
      solution: solution || null,
      minimalDislike,
      score: solution.score || 0,
      allSolutions: filtered
    }
  })

  const rowsPerParams = Object.keys(solutionsForEachParams).map((k: any): TableRowDataForParams => {
    return {
      commitHash: k.split(":")[0],
      params: k.split(":")[1],
      score: solutionsForEachParams[k].reduce((agg: number, next: any) => agg + next.score, 0),
      solved: solutionsForEachParams[k].length,
      allSolutions: solutionsForEachParams[k],
    }
  })
  rowsPerParams.sort((a, b) => b.score - a.score)

  return {
    props: { rows, rowsPerParams }
  }
}

export default function Home({ rows, rowsPerParams }: Props) {
  const classes = useStyles();
  const canvasRefs = useRef([] as RefObject<HTMLCanvasElement>[]);

  if (canvasRefs.current.length !== rows.length) {
    canvasRefs.current = Array(rows.length).fill(null).map((_, i) => canvasRefs.current[i] || createRef());
  }

  useEffect(() => {
    for (let i = 0; i < rows.length; i++) {
      const canvas = canvasRefs.current[i].current;
      if (!canvas) {
        continue;
      }

      const problem = rows[i].problem;
      const solution = rows[i].solution;
      const ctx = canvas.getContext('2d')!;

      let size = -1
      for (let [x, y] of [...problem.hole, ...problem.figure.vertices]) {
        size = Math.max(size, x, y)
      }
      size += 10
      const scale = canvas.width / (size * 1.0)

      ctx.resetTransform()
      ctx.scale(scale, scale)

      ctx.clearRect(0, 0, size, size);
      ctx.fillStyle = "#00000066"
      ctx.fillRect(0, 0, size, size);

      // hole
      ctx.beginPath()
      ctx.moveTo(problem.hole[0][0], problem.hole[0][1])
      for (let i = 1; i < problem.hole.length; i++) {
        ctx.lineTo(problem.hole[i][0], problem.hole[i][1])
      }
      ctx.fillStyle = "#e1ddd1"
      ctx.fill()

      // vertices (problem)
      ctx.beginPath()
      let vertices = problem.figure.vertices
      for (let i = 0; i < problem.figure.edges.length; i++) {
        const [edgeFrom, edgeTo] = problem.figure.edges[i]
        ctx.moveTo(vertices[edgeFrom][0], vertices[edgeFrom][1])
        ctx.lineTo(vertices[edgeTo][0], vertices[edgeTo][1])
      }
      ctx.strokeStyle = "#0000ff50"
      ctx.stroke()

      if (solution) {
        ctx.beginPath()
        vertices = solution.Pose.vertices
        for (let i = 0; i < problem.figure.edges.length; i++) {
          const [edgeFrom, edgeTo] = problem.figure.edges[i]
          ctx.moveTo(vertices[edgeFrom][0], vertices[edgeFrom][1])
          ctx.lineTo(vertices[edgeTo][0], vertices[edgeTo][1])
        }
        ctx.strokeStyle = "#ff0000"
        ctx.stroke()
      }
    }
  }, [rows, canvasRefs]);

  const totalScore = rows.reduce((a, b) => a + b.score, 0)

  return (
    <>
      <Box>
        <Typography variant="h6" gutterBottom component="div">
          Total Score: {totalScore}
        </Typography>
        <CommitParamsTable rows={rowsPerParams} />
        <Typography variant="h6" gutterBottom component="div">
          Each Problems
        </Typography>
        <TableContainer component={Paper}>
          <Table stickyHeader className={classes.table} aria-label="simple table">
            <TableHead>
              <TableRow>
                <TableCell />
                <TableCell>Problem</TableCell>
                <TableCell>CommitHash</TableCell>
                <TableCell>Params</TableCell>
                <TableCell align="right">Dislike</TableCell>
                <TableCell align="right">Miminal Dislike</TableCell>
                <TableCell align="right">Score</TableCell>
                <TableCell>Visualize</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {rows.map((row, i) => (
                <Row key={row.id} row={row} canvasRef={canvasRefs.current[i]} />
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </Box>
    </>
  )
}

const useRowStyles = makeStyles({
  root: {
    '& > *': {
      borderBottom: 'unset',
    },
  },
});

function CommitParamsTable(props: { rows: readonly TableRowDataForParams[] }) {
  const { rows } = props
  return (
    <Fragment>
      <Typography variant="h6" gutterBottom component="div">
        CommitParams Table
      </Typography>
      <TableContainer component={Paper}>
        <Table aria-label="simple table">
          <TableHead>
            <TableRow>
              <TableCell>CommitHash</TableCell>
              <TableCell>Params</TableCell>
              <TableCell align="right">Solved</TableCell>
              <TableCell align="right">Score</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {rows.map((row, i) => (
              <TableRow key={`${row.commitHash}:${row.params}`}>
                <TableCell>{row.commitHash}</TableCell>
                <TableCell>{row.params}</TableCell>
                <TableCell align="right">{row.solved}</TableCell>
                <TableCell align="right">{row.score}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
    </Fragment>
  )
}

function Row(props: { row: any, canvasRef: any }) {
  const { row, canvasRef } = props;
  const [open, setOpen] = useState(false);
  const classes = useRowStyles()
  return (
    <Fragment>
      <TableRow key={row.id} className={classes.root}>
        <IconButton aria-label="expand row" size="small" onClick={() => setOpen(!open)}>
          {open ? <KeyboardArrowUpIcon /> : <KeyboardArrowDownIcon />}
        </IconButton>
        <TableCell component="th" scope="row">
          {row.id}
        </TableCell>
        <TableCell>{row.commitHash}</TableCell>
        <TableCell>{row.params}</TableCell>
        <TableCell align="right">{row.dislike}</TableCell>
        <TableCell align="right">{row.minimalDislike}</TableCell>
        <TableCell align="right">{row.score}</TableCell>
        <TableCell>
          <canvas ref={canvasRef} width={150} height={150}></canvas>
        </TableCell>
      </TableRow>
      <TableRow>
        <TableCell style={{ paddingBottom: 0, paddingTop: 0 }} colSpan={8}>
          <Collapse in={open} timeout="auto" unmountOnExit>
            <Box margin={1}>
              <Typography variant="h6" gutterBottom component="div">
                All Solutions
              </Typography>
              <Table stickyHeader aria-label="simple table">
                <TableHead>
                  <TableRow>
                    <TableCell>CommitHash</TableCell>
                    <TableCell>Params</TableCell>
                    <TableCell align="right">Dislike</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {row.allSolutions.map((r: Solution, i: any) => (
                    <TableRow key={r['Commit:Params']}>
                      <TableCell>{r['Commit:Params'].split(":")[0]}</TableCell>
                      <TableCell>{r['Commit:Params'].split(":")[1]}</TableCell>
                      <TableCell align="right">{r.Dislikes}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </Box>
          </Collapse>
        </TableCell>
      </TableRow>
    </Fragment>
  );
}