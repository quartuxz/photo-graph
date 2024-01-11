

let incorrectCredsOnce =  false;
let initialHtml = document.getElementById("contextInner").innerHTML;


document.getElementById("createLink").href = domainName+"create_account";

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
    let final = await response.text();
    if(final!="fail"){
        setCookie("session",final,2);
        document.getElementById("contextInner").innerHTML = initialHtml;
        window.location.href = domainName;
    }else{
        if(!incorrectCredsOnce){
            document.getElementById("contextInner").insertAdjacentHTML("afterbegin", "USERNAME OR PASSWORD NOT CORRECT!");
            incorrectCredsOnce = true;
        }

    }
}

document.getElementById("loginForm").onsubmit = () => {console.log("first"); loginFormSubmit(); return false;};