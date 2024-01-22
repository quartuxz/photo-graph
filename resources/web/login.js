

let incorrectCredsOnce =  false;
let initialHtml = document.getElementById("contextInner").innerHTML;


document.getElementById("createLink").href = "create_account.html";

async function loginFormSubmit(){
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
    let response = await fetch("/login", options);
    if(response.ok){
        document.getElementById("contextInner").innerHTML = initialHtml;
        window.location.href = "graph_selector.html";
    }else{
        if(!incorrectCredsOnce){
            document.getElementById("contextInner").insertAdjacentHTML("afterbegin", "USERNAME OR PASSWORD NOT CORRECT!");
            incorrectCredsOnce = true;
        }

    }
}

document.getElementById("loginForm").onsubmit = () => {loginFormSubmit(); return false;};