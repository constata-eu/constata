
const authProvider = {
  login: ({ username, password, otp }) => {
    const api_url = `${process.env.REACT_APP_BACK_OFFICE_API || ''}/sessions/`
    const request = new Request(api_url, {
      method: 'POST',
      body: JSON.stringify({
        "username": username,
        "password": password,
        "otp": otp,
      }),
      headers: new Headers({ 'Content-Type': 'text/plain' }),
    });

    return fetch(request)
      .then(response => {
        if (response.status < 200 || response.status >= 300) {
          throw new Error('admin.errors.invalidPassword');
        }
        return Promise.resolve(response.json());
      })
      .then(( session ) => {
        localStorage.setItem('token', session.token);
        localStorage.setItem('username', session.username);
        localStorage.setItem('permissions', session.permissions);
      });
  },
  checkError: (error, graphQLErrors) => {
      
    if (graphQLErrors && graphQLErrors[0].message === "401") {
      localStorage.removeItem('token');
      localStorage.removeItem('permissions');
      localStorage.removeItem('username');
      window.location.reload();
      return Error.status = 401;
    }

    if (error && error.statusCode) {
      const status = error.statusCode;

      if (status === 401 || status === 403) {
        localStorage.removeItem('token');
        localStorage.removeItem('permissions');
        localStorage.removeItem('username');
        window.location.reload();
        return Error.status = 401;
      }
    }
    return true
  },
  getToken: () => { return localStorage.getItem('token'); },
  checkAuth: () => {
    return localStorage.getItem('token')
    ? Promise.resolve()
    : Promise.reject()
  },
  logout: () => {
    localStorage.removeItem('token');
    localStorage.removeItem('permissions');
    localStorage.removeItem('username');
    return Promise.resolve();
  },
  getIdentity: () => {
    try {
      const username = localStorage.getItem('username');
      return Promise.resolve({ username });
    } catch (error) {
      return Promise.reject(error);
    }
  },
  getPermissions: () => {
    const role = localStorage.getItem('permissions');
    return role ? Promise.resolve(role) : Promise.reject();
  }
};

export default authProvider;