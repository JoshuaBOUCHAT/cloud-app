// src/app/auth/verify-account.component.ts
import { Component, OnInit } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { ActivatedRoute } from '@angular/router';
import { CommonModule } from '@angular/common';
import { MatCardModule } from '@angular/material/card';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';

@Component({
  selector: 'app-verify-account',
  standalone: true,
  imports: [CommonModule, MatCardModule, MatProgressSpinnerModule],
  template: `
    <div class="verify-container">
      <mat-card class="verify-card">
        <ng-container *ngIf="loading; else resultTpl">
          <mat-spinner></mat-spinner>
          <p>Vérification en cours...</p>
        </ng-container>
        <ng-template #resultTpl>
          <h2>{{ messageTitle }}</h2>
          <p>{{ messageBody }}</p>
        </ng-template>
      </mat-card>
    </div>
  `,
  styles: [`
    .verify-container {
      display: flex;
      justify-content: center;
      align-items: center;
      min-height: 80vh;
      background: linear-gradient(to bottom, #f3f4f6, #e5e7eb);
    }
    .verify-card {
      padding: 2rem;
      text-align: center;
      max-width: 400px;
      width: 100%;
      border-radius: 1rem;
      box-shadow: 0 8px 20px rgba(0,0,0,0.1);
    }
    mat-spinner {
      margin-bottom: 1rem;
    }
  `]
})
export class VerifyAccountComponent implements OnInit {
  loading = true;
  messageTitle = '';
  messageBody = '';

  constructor(private route: ActivatedRoute, private http: HttpClient) { }

  ngOnInit(): void {
    const token = this.route.snapshot.queryParamMap.get('token');
    if (!token) {
      this.loading = false;
      this.messageTitle = 'Lien invalide';
      this.messageBody = 'Aucun token fourni dans l\'URL.';
      return;
    }

    this.http.post<{ error?: string, code?: number }>('/api/auth/verify', { token })
      .subscribe({
        next: (res: any) => {
          this.loading = false;
          if (res.error) {
            this.messageTitle = 'Erreur';
            this.messageBody = res.error;
          } else {
            this.messageTitle = 'Compte vérifié';
            this.messageBody = 'Votre compte a été validé avec succès !';
          }
        },
        error: (err) => {
          this.loading = false;
          this.messageTitle = 'Erreur serveur';
          this.messageBody = 'Impossible de vérifier le compte pour le moment.';
          console.error(err);
        }
      });
  }
}
