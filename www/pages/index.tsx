import fs from 'fs'
import path from "path";
import Link from 'next/link'
import { Button, ButtonGroup } from '@material-ui/core'

import { makeStyles } from '@material-ui/core/styles';
import Table from '@material-ui/core/Table';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableContainer from '@material-ui/core/TableContainer';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';
import Paper from '@material-ui/core/Paper';

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

const useStyles = makeStyles({
  table: {
    minWidth: 650,
  },
});

interface Props {
  rows: readonly TableRowData[];
}

interface TableRowData {
  id: number;
  dislike: number;
  minimalDislike: number;
  score: number;
  problem: Problem;
  solution: Solution | null;
}

export async function getStaticProps() {
  const dir = path.join(process.cwd(), '..', 'problems');
  const files = fs.readdirSync(dir);
  const ids: number[] = files.map(f => parseInt(path.basename(f, '.problem'), 10));

  const rows = ids.sort().map((id): TableRowData => {
    const problemPath = path.join(process.cwd(), '..', 'problems', `${id}.problem`)
    const problem = JSON.parse(fs.readFileSync(problemPath, 'utf-8')) as Problem;

    const solutionPath = path.join(process.cwd(), '..', 'solutions', `${id}.solution`)
    const solution = fs.existsSync(solutionPath) ?
      JSON.parse(fs.readFileSync(solutionPath, 'utf-8')) as Solution
      : null;

    return {
      id,
      dislike: 0,
      minimalDislike: 0,
      score: 0,
      problem,
      solution,
    };
  })

  return {
    props: { rows }
  }
}

export default function Home({ ids, rows }: Props) {
  const classes = useStyles();

  return (
    <>
      <TableContainer component={Paper}>
        <Table className={classes.table} aria-label="simple table">
          <TableHead>
            <TableRow>
              <TableCell>Problem</TableCell>
              <TableCell align="right">Dislike</TableCell>
              <TableCell align="right">Miminal Dislike</TableCell>
              <TableCell align="right">Score</TableCell>
              <TableCell>Visualize</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {rows.map((row) => (
              <TableRow key={row.id}>
                <TableCell component="th" scope="row">
                  {row.id}
                </TableCell>
                <TableCell align="right">{row.dislike}</TableCell>
                <TableCell align="right">{row.minimalDislike}</TableCell>
                <TableCell align="right">{row.score}</TableCell>
                <TableCell>
                  {/* <canvas width={200} height={200}></canvas> */}
                  {JSON.stringify(row.problem)}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
    </>
  )
}
