let problem = {
    "hole": [[45, 80], [35, 95], [5, 95], [35, 50], [5, 5], [35, 5], [95, 95], [65, 95], [55, 80]], "epsilon": 150000, "figure": { "edges": [[2, 5], [5, 4], [4, 1], [1, 0], [0, 8], [8, 3], [3, 7], [7, 11], [11, 13], [13, 12], [12, 18], [18, 19], [19, 14], [14, 15], [15, 17], [17, 16], [16, 10], [10, 6], [6, 2], [8, 12], [7, 9], [9, 3], [8, 9], [9, 12], [13, 9], [9, 11], [4, 8], [12, 14], [5, 10], [10, 15]], "vertices": [[20, 30], [20, 40], [30, 95], [40, 15], [40, 35], [40, 65], [40, 95], [45, 5], [45, 25], [50, 15], [50, 70], [55, 5], [55, 25], [60, 15], [60, 35], [60, 65], [60, 95], [70, 95], [80, 30], [80, 40]] }
}
let solution = {
    "vertices": [
        [21, 28], [31, 28], [31, 87], [29, 41], [44, 43], [58, 70],
        [38, 79], [32, 31], [36, 50], [39, 40], [66, 77], [42, 29],
        [46, 49], [49, 38], [39, 57], [69, 66], [41, 70], [39, 60],
        [42, 25], [40, 35]
    ]
}


function render(canvas, problem, solution) {
    const ctx = canvas.getContext('2d')
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


    // vertices
    ctx.beginPath()
    const vertices = solution.vertices
    for (let i = 0; i < problem.figure.edges.length; i++) {
        const [edgeFrom, edgeTo] = problem.figure.edges[i]
        ctx.moveTo(vertices[edgeFrom][0], vertices[edgeFrom][1])
        ctx.lineTo(vertices[edgeTo][0], vertices[edgeTo][1])
    }
    ctx.strokeStyle = "#ff0000"
    ctx.stroke()

}

const canvas = document.getElementById("pose")
const problemElem = document.getElementById("problem");
const solutionElem = document.getElementById("solution");
const renderProblemElem = document.getElementById("render_problem");
const renderSolutionElem = document.getElementById("render_solution");

problemElem.addEventListener('change', (event) => {
    const reader = new FileReader();
    reader.onload = ev => {
        problem = JSON.parse(ev.target.result)
    }
    reader.readAsText(event.target.files[0]);
});

solutionElem.addEventListener('change', (event) => {
    const reader = new FileReader();
    reader.onload = ev => {
        solution = JSON.parse(ev.target.result)
    }
    reader.readAsText(event.target.files[0]);
});

renderProblemElem.addEventListener('click', () => {
    render(canvas, problem, { vertices: problem.figure.vertices });
});

renderSolutionElem.addEventListener('click', () => {
    render(canvas, problem, solution)
});

