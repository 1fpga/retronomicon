import React, {FormEvent, useEffect} from 'react';
import './App.css';
import axios from 'axios';

function App() {
  const [username, setUsername] = React.useState<string | null>(null);
  const [description, setDescription] = React.useState<string>("");

  const [token, setToken] = React.useState<string | null>(null);

  async function getUser() {
    return axios.get(process.env.REACT_APP_BACKEND_URL + "/me")
        .then((res) => res.data)
        .catch(() => null)
        .then((user) => {
            setUsername(user?.username);
            setDescription(user?.description || "");
        })
  }
  if (username == null) {
    getUser();
  }

  useEffect(() => {
    let id = setInterval(getUser, 10000);
    return () => clearInterval(id);
  });

  function updateUser(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    let newUser: any = { };
    if (data.get("username_") !== username) {
      newUser.username = data.get("username_");
    }
    if (data.get("description_") !== description) {
      newUser.description = data.get("description_");
    }

    axios.post(process.env.REACT_APP_BACKEND_URL + "/me/update", newUser)
        .then((res) => getUser());
  }

  return (
    <div className="App">
      <header className="App-header">
        <p>
          <a href={process.env.REACT_APP_BACKEND_URL + "/login/google"}>Login using Google</a><br/>
          <a href={process.env.REACT_APP_BACKEND_URL + "/login/github"}>Login using Github</a>
        </p>
        <button onClick={() => axios.get(process.env.REACT_APP_BACKEND_URL + "/logout").then(() => getUser())}>Logout</button>
        <p>
          Username: {username === "" ? "Not yet set" : JSON.stringify(username)}
        </p>

        <hr style={{"width": "100%"}}/>
        <h3>Update user:</h3>
        <form onSubmit={updateUser}>
          <label>Username:</label>
          <input type="text" name="username_" defaultValue={username || ""} /><br/>
          <label>Description:</label>
          <input type="text" name="description_" defaultValue={description} /><br/>

          <button type="submit">Update</button>
        </form>

        <hr style={{"width": "100%"}}/>

        <button onClick={() => axios.post(process.env.REACT_APP_BACKEND_URL + "/me/token").then((res) => setToken(res.data.token))}>Get token</button>
        {token && <>
          <br/><textarea style={{ font: "monospace" }} cols={80}>{token}</textarea><br/><button onClick={() => navigator.clipboard.writeText(token)}>Copy to clipboard</button>
        </>}
      </header>
    </div>
  );
}

export default App;
