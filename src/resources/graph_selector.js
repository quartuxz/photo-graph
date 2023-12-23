
let graphFiles = null;
async function onsubmitCreateForm(){
    let graphFile = document.getElementById("graphName").value + ".graph"
    const options = {
        method: "POST",
        headers: {
            "Content-Type": "text",
          },
        body: graphFile
    };
    let response = await fetch("/createGraph", options);
    let graphID = Number(await response.text());
    
    const options2 = {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({fileName:graphFile,graphID:graphID})
    };
    await fetch("/saveGraph", options2);
    setCookie("graphFile",graphFile,2);
    setCookie("graphID",graphID,2);
    window.location.href = window.location.href.split("/")[0]+"/graph";

}

async function createNew(){
    let contents= "";
    contents += "<form id=\"createForm\" name=\"createForm\" onsubmit=\"onsubmitCreateForm(); return false;\">";
    contents += "<label for=\"graphName\">graph name:</label><br>"
    contents += "<input type=\"text\" id=\"graphName\" name=\"graphName\"></input><br>"
    contents += "<input type=\"submit\" value=\"Create\"></input>  </form>";
    document.getElementById("contextInner").innerHTML = contents;
}

function onsubmitLoadForm(){
    let mainBody = async function(){
        for(const graphFile of graphFiles){
            let fileName = graphFile.split(".")[0];
            if(document.getElementById(fileName).checked){
                const options = {
                    method: "POST",
                    headers: {
                        "Content-Type": "text",
                      },
                    body: graphFile
                };
                let response = await fetch("/loadGraph", options);
                let graphID = Number(await response.text());
                setCookie("graphFile",graphFile,2);
                setCookie("graphID",graphID,2);
                window.location.href = window.location.href.split("/")[0]+"/graph";
            }
        }
    };
    mainBody();
    return false;
}

async function mainMenu(){
    const options = {
        method: "GET"
    };
    let response = await fetch("/retrieveGraphFileList", options);
    graphFiles = await response.json();
    console.log(graphFiles);
    let contents= "<form id=\"loadForm\" name=\"loadForm\">";
    for(const graphFile of graphFiles){
        let fileName = graphFile.split(".")[0];

        contents += "<input type=\"radio\" name=\"fileName\" id=\""+fileName+"\"></input>";
        contents += "<label for=\""+fileName+"\">"+fileName+"</label><br>"
    }
    contents += "<input type=\"submit\" value=\"load\"></input>  </form>";
    contents += "</form>";
    contents += "<button id=\"createButton\" type=\"button\">create new</button>";
    document.getElementById("contextInner").innerHTML = contents;
    document.getElementById("createButton").onclick = createNew;
    document.getElementById("loadForm").onsubmit = onsubmitLoadForm;
}

mainMenu();