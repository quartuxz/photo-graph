

async function createNew(){
    let contents= "";
    contents += "<form id=\"createForm\">";
    contents += "<label for=\"graphName\">graph name:</label><br>"
    contents += "<input type=\"text\" id=\"graphName\" name=\"graphName\"></input><br>"
    contents += "<input type=\"submit\" value=\"Create\"></input>  </form>";
    document.getElementById("contextInner").innerHTML = contents;
    document.getElementById("createForm").onsubmit = function (){
        return false;
    };
}

async function mainMenu(){
    const options = {
        method: "GET"
    };
    let response = await fetch("/retrieveGraphFileList", options);
    let graphFiles = await response.json();
    console.log(graphFiles);
    let contents= "";
    contents += "<button id=\"createButton\" type=\"button\">create new</button>";
    document.getElementById("contextInner").innerHTML = contents;
    document.getElementById("createButton").onclick = createNew;
}

mainMenu();