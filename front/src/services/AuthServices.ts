// src/lib/api.ts

import ThreeColumnImageGrid from "../components/ui/images/ThreeColumnImageGrid";


const app_url = "https://localhost/api"

export const api = {
  async request(method: string, url: string, body?: any) {
    const options: RequestInit = {
      method,
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",  // IMPORTANT si ton refresh token est en session/cookie
    };

    if (body) {
      options.body = JSON.stringify(body);
    }

    let res = await fetch(app_url + url, options);

    // ----------- Case 401 : token expiré → essayer refresh automatiquement ----------
    if (res.status === 401) {
      const refreshed = await this.refreshToken();
      if (refreshed) {
        res = await fetch(url, options); // retry
      }
    }

    // ----------- Toujours parser la réponse ----------
    const data = await res.json().catch(() => null);

    if (!res.ok) {
      throw { status: res.status, data };
    }

    return data;
  },

  get(url: string) {
    return this.request("GET", url);
  },

  post(url: string, body?: any) {
    return this.request("POST", url, body);
  },

  put(url: string, body?: any) {
    return this.request("PUT", url, body);
  },

  delete(url: string) {
    return this.request("DELETE", url);
  },

  // ---------------- Refresh token automatique ----------------
  async refreshToken() {
    const res = await fetch("/auth/refresh_token", {
      method: "POST",
      credentials: "include",
    });

    if (!res.ok) return false;

    const data = await res.json();

    if (data.type === "Token") {
      // ex: USER_NOT_LOGIN
      localStorage.setItem("access_token", data.data);
      console.log("token stored");
      return true;
    }


    // type === Token → success
    // tu peux stocker ton access token si nécessaire


    return false;
  },
  async login(body: any) {
    return this.post("/auth/login", body)
  }

};
