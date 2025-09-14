import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable, tap } from 'rxjs';

export interface LoginPayload {
  email: string;
  password: string;
}

export interface RegisterPayload {
  email: string;
  password: string;
  confirmPassword?: string; // optionnel côté frontend
}

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private login_url = '/api/auth/login';
  private register_url = '/api/auth/register';

  constructor(private http: HttpClient) {}

  /** Login utilisateur */
  login(email: string, password: string): Observable<any> {
    const payload: LoginPayload = { email, password };
    return this.http.post<any>(this.login_url, payload).pipe(
      tap(res => {
        // éventuellement stocker un token/session côté client si nécessaire
        console.log('Login response:', res);
      })
    );
  }

  /** Register utilisateur */
  register(email: string, password: string): Observable<any> {
    const payload: RegisterPayload = { email, password };
    return this.http.post<any>(this.register_url, payload).pipe(
      tap(res => {
        console.log('Register response:', res);
      })
    );
  }
}
