let incorrectCredsOnce =  false;
let initialHtml = document.getElementById("contextInner").innerHTML;



async function createFormSubmit(){
    let username = document.getElementById("username").value;
    let password = document.getElementById("password").value;

    let body = {username:username, password:password};
    const options = {
        method: "POST",
        headers: {
        "Content-Type": "application/json",
        },
        body: JSON.stringify(body)
    };
    let response = await fetch("/createAccount", options);
    let final = await response.text();
    if(final=="ok"){
        document.getElementById("contextInner").innerHTML = initialHtml;
        window.location.href = "login.html";
    }else{
        if(!incorrectCredsOnce){
            document.getElementById("contextInner").insertAdjacentHTML("afterbegin", "USERNAME ALREADY TAKEN OR NOT ALLOWED!");
            incorrectCredsOnce = true;
        }

    }
}

document.getElementById("createForm").onsubmit = () => {createFormSubmit(); return false;};