import { promises as fs } from 'fs'
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

const useStyles = makeStyles({
  table: {
    minWidth: 650,
  },
});

interface Props {
  ids: readonly string[];
}

export async function getStaticProps() {
  const dir = path.join(process.cwd(), '..', 'solutions');
  const files = await fs.readdir(dir);
  const ids = files.map(f => path.basename(f, '.solution'));

  return {
    props: { ids }
  }
}

function createData(name, calories, fat, carbs, protein) {
  return { name, calories, fat, carbs, protein };
}

const rows = [
  createData('Frozen yoghurt', 159, 6.0, 24, 4.0),
  createData('Ice cream sandwich', 237, 9.0, 37, 4.3),
  createData('Eclair', 262, 16.0, 24, 6.0),
  createData('Cupcake', 305, 3.7, 67, 4.3),
  createData('Gingerbread', 356, 16.0, 49, 3.9),
];

export default function Home({ ids }: Props) {
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
              <TableRow key={row.name}>
                <TableCell component="th" scope="row">
                  {row.name}
                </TableCell>
                <TableCell align="right">{row.calories}</TableCell>
                <TableCell align="right">{row.fat}</TableCell>
                <TableCell align="right">{row.carbs}</TableCell>
                <TableCell>
                  <canvas width={200} height={200}></canvas>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
      <div style={{ margin: '0.5em' }}>
        <Button variant="contained">Default</Button>{' '}
        <Button variant="contained" color="primary">Primary</Button>{' '}
        <Button variant="contained" color="secondary">Secondary</Button>{' '}
        <Button variant="contained" disabled>Disabled</Button>{' '}
        <Button variant="contained" color="primary" href="https://google.com/">LINK</Button>
      </div>
      <div style={{ margin: '0.5em' }}>
        <ButtonGroup variant="contained" color="primary" aria-label="contained primary button group">
          <Button>One</Button>
          <Button>Two</Button>
          <Button>Three</Button>
        </ButtonGroup>
      </div>
      <h3>Solutions</h3>
      <ul>
        {
          ids.map(id => <li key={id}><Link href={`/problems/${id}`}><a>{id}</a></Link></li>)
        }
      </ul>
    </>
  )
}
