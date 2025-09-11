import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable, tap } from 'rxjs';

interface LoginResponse {
  access_token: string;
  refresh_token: string;
  expires_in: number;
}

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private login_url = '/api/auth/login';

  constructor(private http: HttpClient) { }

  login(email: string, password: string): Observable<LoginResponse> {
    return this.http.post<LoginResponse>(this.login_url, {
      email: email,
      password: password
    }).pipe(
      tap(res => {
        // stocker les tokens dans localStorage/sessionStorage
        localStorage.setItem('access_token', res.access_token);
        localStorage.setItem('refresh_token', res.refresh_token);
        // éventuellement stocker l'expiration pour gérer le refresh automatique
        localStorage.setItem('access_token_exp',
          (Date.now() + res.expires_in * 1000).toString()
        );
      })
    );
  }
  login_test(): Observable<string> {
    const token = localStorage.getItem('access_token');
  
    return this.http.post<string>(
      "/api/login_test",
      {}
    ).pipe(
      tap(res => {
        console.log(res);
      })
    );
  }
  getAccessToken(): string | null {
    return localStorage.getItem('access_token');
  }

  getRefreshToken(): string | null {
    return localStorage.getItem('refresh_token');
  }

  logout() {
    localStorage.removeItem('access_token');
    localStorage.removeItem('refresh_token');
    localStorage.removeItem('access_token_exp');
  }
}
