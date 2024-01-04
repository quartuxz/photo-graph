
function saveOnclick(){

  const options = {
      method: "POST",
      headers: {
          "Content-Type": "application/json",
      },
      body: JSON.stringify({fileName:graphFile,graphID:graphID})
  };
  fetch("/saveGraph", options);
  return true;
}


async function main(){
  
  await GraphNode.loadNodeTemplates();
  document.getElementById("saveButton").onclick = saveOnclick;


  let canvas = document.getElementById("canvas");
  let context = canvas.getContext("2d")
  let graph = new Graph(context);

  const options = {
    method: "POST",
    headers: {
      "Content-Type": "text",
    },
    body: graphID.toString()
  };
  let response = await fetch("/retrieveGraph", options);
  let commandHistory = await response.json();
  if(commandHistory.isValid == "no" || graphFile == ""){
    window.location.href = domainName;
  }
  graph.interpretCommands(commandHistory);

  let ui = new UI(graph,canvas, context);
}

main();
