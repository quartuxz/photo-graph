
console.log(getCookie("session"));

let graphFiles = null;
async function onsubmitCreateForm(){
    let graphFile = document.getElementById("graphName").value
    const options = {
        method: "POST",
        headers: {
            "Content-Type": "text",
          },
        body: graphFile
    };
    let response = await fetch("/createGraph", options);
    if(response.status==401){window.location.href = domainName+"login";}
    let graphID = Number(await response.text());
    
    const options2 = {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({fileName:graphFile,graphID:graphID})
    };
    let response2 = await fetch("/saveGraph", options2);
    if(response2.status==401){window.location.href = domainName+"login";}
    setCookie("graphFile",graphFile,2);
    setCookie("graphID",graphID,2);
    
    window.location.href = domainName+"graph";

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
            console.log(graphFile);
            if(document.getElementById(graphFile).checked){
                const options = {
                    method: "POST",
                    headers: {
                        "Content-Type": "text",
                      },
                    body: graphFile
                };
                let response = await fetch("/loadGraph", options);
                if(response.status==401){window.location.href = domainName+"login";}
                let graphID = Number(await response.text());
                setCookie("graphFile",graphFile,2);
                setCookie("graphID",graphID,2);
                window.location.href = domainName+"graph";
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
    //credentials are no longer valid
    if(response.status==401){window.location.href = domainName+"login";}
    graphFiles = await response.json();
    let contents= "<form id=\"loadForm\" name=\"loadForm\">";
    for(const graphFile of graphFiles){
        contents += "<input type=\"radio\" name=\"fileName\" id=\""+graphFile+"\"></input>";
        contents += "<label for=\""+graphFile+"\">"+graphFile+"</label><br>"
    }
    contents += "<input type=\"submit\" value=\"load\"></input>  </form>";
    contents += "</form>";
    contents += "<button id=\"createButton\" type=\"button\">create new</button>";
    document.getElementById("contextInner").innerHTML = contents;
    document.getElementById("createButton").onclick = createNew;
    document.getElementById("loadForm").onsubmit = onsubmitLoadForm;
}

mainMenu();