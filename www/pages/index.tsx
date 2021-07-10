import { promises as fs } from 'fs'
import path from "path";
import Link from 'next/link'
import { Button, ButtonGroup } from '@material-ui/core'

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

export default function Home({ ids }: Props) {
  return (
    <>
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
