import { Component, OnInit, NgZone } from '@angular/core';
import { CommonModule } from '@angular/common';
import { PingService } from './services/ping.service';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule],
  template: `
    <h1>Test API Ping</h1>
    <button (click)="doPing()">Re-ping</button>

    <p *ngIf="response; else loading">{{ response | json }} hello</p>
    <ng-template #loading>
      <p>Chargement...</p>
    </ng-template>
  `
})
export class AppComponent implements OnInit {
  response: any = null;

  constructor(private pingService: PingService, private ngZone: NgZone) {}

  ngOnInit() {
    this.doPing();
  }

  doPing() {
    this.response = null;
    this.pingService.ping().subscribe({
      next: res => this.ngZone.run(() => this.response = res),
      error: err => this.ngZone.run(() => {
        console.error('Erreur complète :', err); // pour debugger
        this.response = 'Erreur : impossible de joindre le serveur → ' + (err.message || JSON.stringify(err));
      })
      
    });
  }
}
