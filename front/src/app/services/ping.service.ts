import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';

@Injectable({
  providedIn: 'root'
})
export class PingService {
  private apiUrl = '/api/ping';

  constructor(private http: HttpClient) {}

  ping(): Observable<any> {
    return this.http.get(this.apiUrl);
  }
}