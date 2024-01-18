
let lastResponse = window.location.href.split("?");
if(lastResponse.length > 1){
    if(lastResponse[1] == "bad_name"){
        document.getElementById("contextInner").insertAdjacentHTML("afterbegin","The name you selected is not allowed!<br>");
    }
    if(lastResponse[1] == "bad_image"){
        document.getElementById("contextInner").insertAdjacentHTML("afterbegin","The file you selected is not allowed!<br>");
    }
}