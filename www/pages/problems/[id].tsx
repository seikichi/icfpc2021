import { promises as fs } from 'fs'
import path from 'path'
import { useRef, useEffect } from "react";

interface Props {
    id: string;
    problem: any;
    solution: any;
}

export async function getStaticPaths() {
    const dir = path.join(process.cwd(), '..', 'solutions');
    const files = await fs.readdir(dir);

    return {
        paths: files.map(name => {
            const id = path.basename(name, '.solution');
            return { params: { id } };
        }),
        fallback: false,
    }
}

export async function getStaticProps({ params: { id } }: { params: { id: string; } }) {
    const problemPath = path.join(process.cwd(), '..', 'problems', `${id}.problem`);
    const solutionPath = path.join(process.cwd(), '..', 'solutions', `${id}.solution`);

    const problem = JSON.parse(await fs.readFile(problemPath, 'utf-8'));
    const solution = JSON.parse(await fs.readFile(solutionPath, 'utf-8'));

    return { props: { id, problem, solution } }
}

export default function Problem({ id, problem, solution }: Props) {
    const canvasRef = useRef(null as HTMLCanvasElement | null)
    const SIZE = 300;

    useEffect(() => {
        if (!canvasRef) {
            return;
        }
        const canvas = canvasRef.current;

        if (!canvas) {
            return
        }

        const ctx = canvas.getContext('2d')!;

        let size = -1
        for (let [x, y] of [...problem.hole, ...problem.figure.vertices]) {
            size = Math.max(size, x, y)
        }
        size += 10
        const scale = canvas.width / (size * 1.0)

        ctx.resetTransform()
        ctx.scale(scale, scale)

        ctx.clearRect(0, 0, canvas.width, canvas.height);
        ctx.fillStyle = "#00000066"
        ctx.fillRect(0, 0, canvas.width, canvas.height);

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

        ctx.beginPath()
        vertices = solution.vertices
        for (let i = 0; i < problem.figure.edges.length; i++) {
            const [edgeFrom, edgeTo] = problem.figure.edges[i]
            ctx.moveTo(vertices[edgeFrom][0], vertices[edgeFrom][1])
            ctx.lineTo(vertices[edgeTo][0], vertices[edgeTo][1])
        }
        ctx.strokeStyle = "#ff0000"
        ctx.stroke()

    }, [canvasRef, problem, solution]);

    return <canvas ref={canvasRef} width={SIZE} height={SIZE}></canvas>
}
