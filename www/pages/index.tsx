import fs from 'fs'
import path from "path";
import { useRef, createRef, useEffect, RefObject } from "react";
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

import AWS from "aws-sdk"

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
  AWS.config.update({ region: 'ap-northeast-1' })
  const docClient = new AWS.DynamoDB.DocumentClient()
  const TableName = "Problems"
  const problems = await docClient.scan({ TableName }).promise()

  const rows = problems.Items?.sort((a, b) => parseInt(a.ProblemId) - parseInt(b.ProblemId)).map((item): TableRowData => {
    return {
      id: item.ProblemId,
      dislike: item.Dislikes || -1,
      problem: item.Problem as Problem,
      solution: item.Solution ? item.Solution as Solution : null,
      minimalDislike: 0,
      score: 0,
    }
  })

  return {
    props: { rows }
  }
}

export default function Home({ rows }: Props) {
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
        vertices = solution.vertices
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
            {rows.map((row, i) => (
              <TableRow key={row.id}>
                <TableCell component="th" scope="row">
                  {row.id}
                </TableCell>
                <TableCell align="right">{row.dislike}</TableCell>
                <TableCell align="right">{row.minimalDislike}</TableCell>
                <TableCell align="right">{row.score}</TableCell>
                <TableCell>
                  <canvas ref={canvasRefs.current[i]} width={150} height={150}></canvas>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
    </>
  )
}
