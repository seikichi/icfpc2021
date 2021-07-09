import { promises as fs } from 'fs'
import path from "path";
import Link from 'next/link'

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
      <h3>Solutions</h3>
      <ul>
        {
          ids.map(id => <li key={id}><Link href={`/problems/${id}`}><a>{id}</a></Link></li>)
        }
      </ul>
    </>
  )
}
