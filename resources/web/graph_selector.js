

let graphFiles = null;
async function onsubmitCreateForm(){
    let graphName = document.getElementById("graphName").value
    const options = {
        method: "POST",
        body: graphName
    };
    let response = await fetch("/createGraph", options);
    if(response.status==401){window.location.href = "login.html";}
    
    setCookie("graphName",graphName,9999);

    window.location.href = "main.html";

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
                setCookie("graphName",graphFile,9999);
                window.location.href = "main.html";
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
    if(response.status==401){window.location.href = "login";}
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